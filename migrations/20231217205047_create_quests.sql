CREATE TABLE `quests` (
  `id` int NOT NULL,
  `character_id` int NOT NULL,
  `status` enum('NotStarted','Started','Completed') NOT NULL,
  `time` int NOT NULL,
  `expires` bigint NOT NULL,
  `forfeited` int NOT NULL,
  `completed` int NOT NULL,
  `info` int NOT NULL,
  PRIMARY KEY (`id`, `character_id`)
) ENGINE=InnoDB;