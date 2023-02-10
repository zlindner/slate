use crate::maple::Character;
use nx::GenericNode;
use prisma_client_rust::chrono::{NaiveDateTime, Utc};
use std::collections::{HashMap, HashSet};

pub trait Requirement {
    fn has_requirement(&self, character: Character) -> bool;
}

/// Loads the quest start/complete requirements from the given root node
pub fn load_quest_requirements(quest_requirements_root: nx::Node) -> Vec<QuestRequirementType> {
    use QuestRequirementType::*;
    let mut quest_requirements = Vec::new();

    for requirement in quest_requirements_root.iter() {
        quest_requirements.push(match requirement.name() {
            "job" => Job(JobRequirement::new(requirement)),
            "quest" => Quest(QuestRequirement::new(requirement)),
            "item" => Item(ItemRequirement::new(requirement)),
            "lvmin" => MinLevel(MinLevelRequirement::new(requirement)),
            "lvmax" => MaxLevel(MaxLevelRequirement::new(requirement)),
            "end" => EndDate(EndDateRequirement::new(requirement)),
            "mob" => Mob(MobRequirement::new(requirement)),
            "npc" => Npc(NpcRequirement::new(requirement)),
            "fieldEnter" => FieldEnter(FieldEnterRequirement::new(requirement)),
            "interval" => Interval(IntervalRequirement::new(requirement)),
            "startscript" => Script(ScriptRequirement::new(requirement)),
            "endscript" => Script(ScriptRequirement::new(requirement)),
            "pet" => Pet(PetRequirement::new(requirement)),
            "pettamenessmin" => PetTameness(PetTamenessRequirement::new(requirement)),
            "mbmin" => MonsterBook(MonsterBookRequirement::new(requirement)),
            "infoNumber" => InfoNumber(InfoNumberRequirement::new(requirement)),
            "infoex" => InfoExpected(InfoExpectedRequirement::new(requirement)),
            "questComplete" => CompletedQuest(CompletedQuestRequirement::new(requirement)),
            "money" => Meso(MesoRequirement::new(requirement)),
            "buff" => Buff(BuffRequirement::new(requirement)),
            _ => QuestRequirementType::Undefined(requirement.name().to_string()),
        });
    }

    quest_requirements
}

#[derive(Debug)]
pub enum QuestRequirementType {
    Undefined(String),
    Job(JobRequirement),
    Quest(QuestRequirement),
    Item(ItemRequirement),
    MinLevel(MinLevelRequirement),
    MaxLevel(MaxLevelRequirement),
    EndDate(EndDateRequirement),
    Mob(MobRequirement),
    Npc(NpcRequirement),
    FieldEnter(FieldEnterRequirement),
    Interval(IntervalRequirement),
    Script(ScriptRequirement),
    Pet(PetRequirement),
    PetTameness(PetTamenessRequirement),
    MonsterBook(MonsterBookRequirement),
    InfoNumber(InfoNumberRequirement),
    InfoExpected(InfoExpectedRequirement),
    CompletedQuest(CompletedQuestRequirement),
    Meso(MesoRequirement),
    Buff(BuffRequirement),
}

#[derive(Debug)]
pub struct JobRequirement {
    job_ids: Vec<i32>,
}

impl JobRequirement {
    pub fn new(data: nx::Node) -> Self {
        let mut job_ids = Vec::new();

        for job_id in data.iter() {
            job_ids.push(job_id.integer().unwrap() as i32);
        }

        Self { job_ids }
    }
}

impl Requirement for JobRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        if character.data.gm > 1 {
            return true;
        }

        for job_id in self.job_ids.iter() {
            // TODO not sure if we are comparing the right ids here
            if character.data.job == *job_id {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct QuestRequirement {
    quests: HashMap<i32, i32>,
}

impl QuestRequirement {
    pub fn new(data: nx::Node) -> Self {
        let mut quests = HashMap::new();

        for quest in data.iter() {
            let id = quest.get("id").integer().unwrap() as i32;
            let state = quest.get("state").integer().unwrap() as i32;
            quests.insert(id, state);
        }

        Self { quests }
    }
}

impl Requirement for QuestRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct ItemRequirement {
    items: HashMap<i32, i32>,
}

impl ItemRequirement {
    pub fn new(data: nx::Node) -> Self {
        let mut items = HashMap::new();

        for item in data.iter() {
            let id = item.get("id").integer().unwrap() as i32;
            let amount = item.get("count").integer().unwrap() as i32;
            items.insert(id, amount);
        }

        Self { items }
    }
}

impl Requirement for ItemRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct MinLevelRequirement {
    level: i32,
}

impl MinLevelRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            level: data.integer().unwrap_or(0) as i32,
        }
    }
}

impl Requirement for MinLevelRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        character.data.level >= self.level
    }
}

#[derive(Debug)]
pub struct MaxLevelRequirement {
    level: i32,
}

impl MaxLevelRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            level: data.integer().unwrap_or(0) as i32,
        }
    }
}

impl Requirement for MaxLevelRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        character.data.level <= self.level
    }
}

#[derive(Debug)]
pub struct EndDateRequirement {
    end_date: NaiveDateTime,
}

impl EndDateRequirement {
    pub fn new(data: nx::Node) -> Self {
        let date_str = format!("{}0000", data.string().unwrap());
        let end_date = NaiveDateTime::parse_from_str(&date_str, "%Y%m%d%H%M%S").unwrap();
        Self { end_date }
    }
}

impl Requirement for EndDateRequirement {
    fn has_requirement(&self, _: Character) -> bool {
        self.end_date <= Utc::now().naive_utc()
    }
}

#[derive(Debug)]
pub struct MobRequirement {
    mobs: HashMap<i32, i32>,
}

impl MobRequirement {
    pub fn new(data: nx::Node) -> Self {
        let mut mobs = HashMap::new();

        for mob in data.iter() {
            let id = mob.get("id").integer().unwrap() as i32;
            let count = mob.get("count").integer().unwrap() as i32;
            mobs.insert(id, count);
        }

        Self { mobs }
    }
}

impl Requirement for MobRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        // TODO check characters progress for the current quest id
        todo!()
    }
}

#[derive(Debug)]
pub struct NpcRequirement {
    npc_id: i32,
}

impl NpcRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            npc_id: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for NpcRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct FieldEnterRequirement {
    map_id: i32,
}

impl FieldEnterRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            map_id: data.integer().unwrap_or(-1) as i32,
        }
    }
}

impl Requirement for FieldEnterRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        character.map_id == self.map_id
    }
}

#[derive(Debug)]
pub struct IntervalRequirement {
    interval: i32,
}

impl IntervalRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            interval: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for IntervalRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        // TODO check if character has completed the current quest less than self.interval time ago
        // self.interval contains number of minutes character needs to wait after completing quest
        todo!()
    }
}

#[derive(Debug)]
pub struct ScriptRequirement {
    script: String,
}

impl ScriptRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            script: data.string().unwrap().to_string(),
        }
    }
}

impl Requirement for ScriptRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct PetRequirement {
    pet_ids: HashSet<i32>,
}

impl PetRequirement {
    pub fn new(data: nx::Node) -> Self {
        let mut pet_ids = HashSet::new();

        for pet in data.iter() {
            let id = pet.get("id").integer().unwrap() as i32;
            pet_ids.insert(id);
        }

        Self { pet_ids }
    }
}

impl Requirement for PetRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct PetTamenessRequirement {
    min_tameness: i32,
}

impl PetTamenessRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            min_tameness: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for PetTamenessRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct MonsterBookRequirement {
    required_cards: i32,
}

impl MonsterBookRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            required_cards: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for MonsterBookRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct InfoNumberRequirement {
    info_number: i32,
}

impl InfoNumberRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            info_number: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for InfoNumberRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct InfoExpectedRequirement {
    info_expected: Vec<String>,
}

impl InfoExpectedRequirement {
    pub fn new(data: nx::Node) -> Self {
        let mut info_expected = Vec::new();

        for info in data.iter() {
            let value = info.get("value").string().unwrap().to_string();
            info_expected.push(value);
        }

        Self { info_expected }
    }
}

impl Requirement for InfoExpectedRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct CompletedQuestRequirement {
    required_quests: i32,
}

impl CompletedQuestRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            required_quests: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for CompletedQuestRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        // TODO Check character's number of completed quests
        todo!()
    }
}

#[derive(Debug)]
pub struct MesoRequirement {
    mesos: i32,
}

impl MesoRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            mesos: data.integer().unwrap() as i32,
        }
    }
}

impl Requirement for MesoRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct BuffRequirement {
    buff_id: i32,
}

impl BuffRequirement {
    pub fn new(data: nx::Node) -> Self {
        Self {
            buff_id: data.integer().unwrap() as i32 * -1,
        }
    }
}

impl Requirement for BuffRequirement {
    fn has_requirement(&self, character: Character) -> bool {
        todo!()
    }
}
