use super::{QuestActionType, QuestRequirementType};
use crate::maple;
use anyhow::anyhow;
use nx::GenericNode;

#[derive(Debug)]
pub struct Quest {
    name: String,
    parent: String,
    time_limit: i32,
    time_limit2: i32,
    auto_start: bool,
    auto_pre_complete: bool,
    auto_complete: bool,
    repeatable: bool,
    start_requirements: Vec<QuestRequirementType>,
    complete_requirements: Vec<QuestRequirementType>,
    start_actions: Vec<QuestActionType>,
    complete_actions: Vec<QuestActionType>,
}

impl Quest {
    pub fn load(id: i16) -> anyhow::Result<Self> {
        let root = super::DATA.get("Quest").unwrap().root();

        // Load quest info
        let quest_info = root.get("QuestInfo.img").get(&id.to_string());

        if quest_info.is_none() {
            return Err(anyhow!("No info found for quest{}", id));
        }

        let name = quest_info.get("name").string().unwrap_or("").to_string();
        let parent = quest_info.get("parent").string().unwrap_or("").to_string();
        let time_limit = quest_info.get("timeLimit").integer().unwrap_or(0) as i32;
        let time_limit2 = quest_info.get("timeLimit2").integer().unwrap_or(0) as i32;
        let auto_start = quest_info.get("autoStart").integer().unwrap_or(0) == 1;
        let auto_pre_complete = quest_info.get("autoPreComplete").integer().unwrap_or(0) == 1;
        let auto_complete = quest_info.get("autoComplete").integer().unwrap_or(0) == 1;
        // TODO medal_id

        let requirements_root = root.get("Check.img").get(&id.to_string());

        let start_requirements = match requirements_root.get("0") {
            Some(start_requirements_root) => {
                QuestRequirementType::load_all(start_requirements_root)
            }
            None => Vec::new(),
        };

        let repeatable = start_requirements
            .iter()
            .any(|req| matches!(req, QuestRequirementType::Interval(_)));

        // TODO relevant mobs?

        let complete_requirements = match requirements_root.get("1") {
            Some(complete_requirements_root) => {
                QuestRequirementType::load_all(complete_requirements_root)
            }
            None => Vec::new(),
        };

        let actions_root = root.get("Act.img").get(&id.to_string());

        let start_actions = match actions_root.get("0") {
            Some(start_actions_root) => QuestActionType::load_all(start_actions_root),
            None => Vec::new(),
        };

        let complete_actions = match actions_root.get("1") {
            Some(complete_actions_root) => QuestActionType::load_all(complete_actions_root),
            None => Vec::new(),
        };

        Ok(Self {
            name,
            parent,
            time_limit,
            time_limit2,
            auto_start,
            auto_pre_complete,
            auto_complete,
            repeatable,
            start_requirements,
            complete_requirements,
            start_actions,
            complete_actions,
        })
    }

    // Starts the quest
    pub fn start(&self, character: &maple::Character, npc_id: i32) -> bool {
        if !self.can_start(character, npc_id) {
            return false;
        }

        // First check if we can execute all the quest's start actions
        for start_action in self.start_actions.iter() {
            if !start_action.can_execute(character, None) {
                return false;
            }
        }

        // Execute the quest's start actions
        for start_action in self.start_actions.iter() {
            start_action.execute(character, None);
        }

        // TODO TOT mob quest requirement?
        // TODO get characters current progress for the quest
        true
    }

    pub fn complete(
        &self,
        character: &maple::Character,
        npc_id: i32,
        selection: Option<i16>,
    ) -> bool {
        if !self.can_complete(character, npc_id) {
            return false;
        }

        // TODO check info progress

        // First check if we can execute all the quest's complete actions
        for complete_action in self.complete_actions.iter() {
            if !complete_action.can_execute(character, selection) {
                return false;
            }
        }

        if self.time_limit > 0 {
            // TODO chr.sendPacket(PacketCreator.removeQuestTimeLimit(id));
        }

        // Create completed quest status
        // QuestStatus newStatus = new QuestStatus(this, QuestStatus.Status.COMPLETED, npc);
        // newStatus.setForfeited(chr.getQuest(this).getForfeited());
        // newStatus.setCompleted(chr.getQuest(this).getCompleted());
        // newStatus.setCompletionTime(System.currentTimeMillis());
        // chr.updateQuestStatus(newStatus);

        // Execute the quest's complete actions
        for complete_action in self.complete_actions.iter() {
            complete_action.execute(character, selection);
        }

        true
    }

    /// Checks if a character can start the current quest
    fn can_start(&self, character: &maple::Character, npc_id: i32) -> bool {
        // TODO check character quest status
        // TODO check if npc is nearby

        for req in self.start_requirements.iter() {
            if !req.has_requirement(character, npc_id) {
                return false;
            }
        }

        // TODO check quest / char info progress

        true
    }

    // Checks if a character can complete the current quest
    fn can_complete(&self, character: &maple::Character, npc_id: i32) -> bool {
        // TODO verify character's quest status is started
        // TODO check if npc is nearby

        for complete_req in self.complete_requirements.iter() {
            if !complete_req.has_requirement(character, npc_id) {
                return false;
            }
        }

        true
    }
}
