CREATE TABLE `login_sessions` (
  `id` int NOT NULL,
  `account_id` int NOT NULL,
  `character_id` int NOT NULL,
  `world_id` int NOT NULL,
  `channel_id` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;