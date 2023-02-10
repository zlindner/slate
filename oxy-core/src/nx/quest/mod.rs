use self::{
    action::{load_quest_actions, QuestActionType},
    requirement::{load_quest_requirements, QuestRequirementType},
};
use nx::GenericNode;

pub mod action;
pub mod requirement;

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

///
pub fn load_quest(quest_id: i32) -> Option<Quest> {
    let root = super::DATA.get("Quest").unwrap().root();

    // Load quest info
    let quest_info = root.get("QuestInfo.img").get(&quest_id.to_string());

    if quest_info.is_none() {
        log::warn!("No quest info found for id: {}", quest_id);
        return None;
    }

    let name = quest_info.get("name").string().unwrap_or("").to_string();
    let parent = quest_info.get("parent").string().unwrap_or("").to_string();
    let time_limit = quest_info.get("timeLimit").integer().unwrap_or(0) as i32;
    let time_limit2 = quest_info.get("timeLimit2").integer().unwrap_or(0) as i32;
    let auto_start = quest_info.get("autoStart").integer().unwrap_or(0) == 1;
    let auto_pre_complete = quest_info.get("autoPreComplete").integer().unwrap_or(0) == 1;
    let auto_complete = quest_info.get("autoComplete").integer().unwrap_or(0) == 1;
    // TODO medal_id

    let requirements_root = root.get("Check.img").get(&quest_id.to_string());

    let start_requirements = match requirements_root.get("0") {
        Some(start_requirements_root) => load_quest_requirements(start_requirements_root),
        None => Vec::new(),
    };

    let repeatable = start_requirements.iter().any(|req| match req {
        QuestRequirementType::Interval(_) => true,
        _ => false,
    });

    // TODO relevant mobs?

    let complete_requirements = match requirements_root.get("1") {
        Some(complete_requirements_root) => load_quest_requirements(complete_requirements_root),
        None => Vec::new(),
    };

    let actions_root = root.get("Act.img").get(&quest_id.to_string());

    let start_actions = match actions_root.get("0") {
        Some(start_actions_root) => load_quest_actions(start_actions_root),
        None => Vec::new(),
    };

    let complete_actions = match actions_root.get("1") {
        Some(complete_actions_root) => load_quest_actions(complete_actions_root),
        None => Vec::new(),
    };

    Some(Quest {
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
