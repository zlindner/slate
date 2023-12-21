use crate::maple;
use nx::GenericNode;
use sqlx::types::chrono::{NaiveDateTime, Utc};
use std::collections::{HashMap, HashSet};

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

impl QuestRequirementType {
    /// Loads the quest start/complete requirements from the root node
    pub fn load_all(root: nx::Node) -> Vec<Self> {
        use QuestRequirementType::*;
        let mut quest_requirements = Vec::new();

        for requirement in root.iter() {
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

    pub fn has_requirement(&self, character: &maple::Character, npc_id: i32) -> bool {
        use QuestRequirementType::*;

        match self {
            Undefined(_) => true,
            Job(req) => req.has_requirement(character),
            Quest(req) => req.has_requirement(character),
            Item(req) => req.has_requirement(character),
            MinLevel(req) => req.has_requirement(character),
            MaxLevel(req) => req.has_requirement(character),
            EndDate(req) => req.has_requirement(),
            Mob(req) => req.has_requirement(character),
            Npc(req) => req.has_requirement(npc_id),
            FieldEnter(req) => req.has_requirement(character),
            Interval(req) => req.has_requirement(character),
            Script(req) => req.has_requirement(character),
            Pet(req) => req.has_requirement(character),
            PetTameness(req) => req.has_requirement(character),
            MonsterBook(req) => req.has_requirement(character),
            InfoNumber(_) => true,
            InfoExpected(_) => true,
            CompletedQuest(req) => req.has_requirement(character),
            Meso(req) => req.has_requirement(character),
            Buff(req) => req.has_requirement(character),
        }
    }
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, npc_id: i32) -> bool {
        self.npc_id == npc_id
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
        character.data.map == self.map_id
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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

    fn has_requirement(&self, character: &maple::Character) -> bool {
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
            buff_id: -(data.integer().unwrap() as i32),
        }
    }

    fn has_requirement(&self, character: &maple::Character) -> bool {
        todo!()
    }
}
