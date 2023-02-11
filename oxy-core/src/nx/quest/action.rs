use crate::maple::Character;
use nx::GenericNode;
use std::collections::HashSet;

#[derive(Debug)]
pub enum QuestActionType {
    Undefined(String),
    Exp(ExpAction),
    Meso(MesoAction),
    Item(ItemAction),
    Skill(SkillAction),
    NextQuest(NextQuestAction),
    Fame(FameAction),
    Buff(BuffAction),
    PetSkill(PetSkillAction),
    PetTameness(PetTamenessAction),
    PetSpeed(PetSpeedAction),
    Info(InfoAction),
}

impl QuestActionType {
    pub fn execute(&self, character: &Character) {
        use QuestActionType::*;

        match self {
            Undefined(_) => todo!(),
            Exp(_) => todo!(),
            Meso(_) => todo!(),
            Item(_) => todo!(),
            Skill(_) => todo!(),
            NextQuest(_) => todo!(),
            Fame(_) => todo!(),
            Buff(_) => todo!(),
            PetSkill(_) => todo!(),
            PetTameness(_) => todo!(),
            PetSpeed(_) => todo!(),
            Info(_) => todo!(),
        }
    }
}

///
pub fn load_quest_actions(quest_actions_root: nx::Node) -> Vec<QuestActionType> {
    use QuestActionType::*;
    let mut quest_actions = Vec::new();

    for action in quest_actions_root.iter() {
        quest_actions.push(match action.name() {
            "exp" => Exp(ExpAction::new(action)),
            "money" => Meso(MesoAction::new(action)),
            "item" => Item(ItemAction::new(action)),
            "skill" => Skill(SkillAction::new(action)),
            "nextQuest" => NextQuest(NextQuestAction::new(action)),
            "pop" => Fame(FameAction::new(action)),
            "buffItemID" => Buff(BuffAction::new(action)),
            "petSkill" => PetSkill(PetSkillAction::new(action)),
            "pettameness" => PetTameness(PetTamenessAction::new(action)),
            "petspeed" => PetSpeed(PetSpeedAction),
            "info" => Info(InfoAction::new(action)),
            _ => Undefined(action.name().to_string()),
        });
    }

    quest_actions
}

#[derive(Debug)]
pub struct ExpAction {
    exp: i32,
}

impl ExpAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            exp: data.integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct MesoAction {
    mesos: i32,
}

impl MesoAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            mesos: data.integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct ItemAction {
    items: Vec<ItemActionEntry>,
}

impl ItemAction {
    pub fn new(data: nx::Node) -> Self {
        let mut items = Vec::new();

        for item in data.iter() {
            items.push(ItemActionEntry {
                name: item.name().to_string(),
                id: item.get("id").integer().unwrap() as i32,
                count: item.get("count").integer().unwrap() as i32,
                period: item.get("period").integer().unwrap() as i32,
                prop: item.get("prop").integer().unwrap_or(-1) as i32,
                gender: item.get("gender").integer().unwrap_or(2) as i32,
                job: item.get("job").integer().unwrap_or(-1) as i32,
            });
        }

        Self { items }
    }
}

#[derive(Debug)]
struct ItemActionEntry {
    name: String,
    id: i32,
    count: i32,
    period: i32,
    prop: i32,
    gender: i32,
    job: i32,
}

#[derive(Debug)]
pub struct SkillAction {
    skills: Vec<SkillActionEntry>,
}

impl SkillAction {
    pub fn new(data: nx::Node) -> Self {
        let mut skills = Vec::new();

        for skill in data.iter() {
            let mut applicable_jobs = HashSet::new();
            let applicable_jobs_root = skill.get("job");

            if applicable_jobs_root.is_some() {
                for job in applicable_jobs_root.unwrap().iter() {
                    applicable_jobs.insert(job.integer().unwrap() as i32);
                }
            }

            skills.push(SkillActionEntry {
                id: skill.get("id").integer().unwrap() as i32,
                level: skill.get("skillLevel").integer().unwrap_or(0) as i32,
                mastery_level: skill.get("masterLevel").integer().unwrap_or(0) as i32,
                applicable_jobs,
            });
        }

        Self { skills }
    }
}

#[derive(Debug)]
struct SkillActionEntry {
    id: i32,
    level: i32,
    mastery_level: i32,
    applicable_jobs: HashSet<i32>,
}

#[derive(Debug)]
pub struct NextQuestAction {
    next_quest_id: i32,
}

impl NextQuestAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            next_quest_id: data.integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct FameAction {
    fame: i32,
}

impl FameAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            fame: data.integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct BuffAction {
    item_effect: i32,
}

impl BuffAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            item_effect: data.integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct PetSkillAction {
    flag: i32,
}

impl PetSkillAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            flag: data.get("petskill").integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct PetTamenessAction {
    tameness: i32,
}

impl PetTamenessAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            tameness: data.integer().unwrap() as i32,
        }
    }
}

#[derive(Debug)]
pub struct PetSpeedAction;

#[derive(Debug)]
pub struct InfoAction {
    info: String,
}

impl InfoAction {
    pub fn new(data: nx::Node) -> Self {
        Self {
            info: data.string().unwrap().to_string(),
        }
    }
}
