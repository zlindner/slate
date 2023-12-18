CREATE TABLE `cooldowns` (
  `character_id` int NOT NULL,
  `skill_id` int NOT NULL,
  `start` bigint NOT NULL,
  `length` bigint NOT NULL,
  PRIMARY KEY (`character_id`, `skill_id`)
) ENGINE=InnoDB;