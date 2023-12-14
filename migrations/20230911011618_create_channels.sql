CREATE TABLE `channels` (
  `id` int NOT NULL,
  `world_name` varchar(32) NOT NULL,
  `world_id` int NOT NULL,
  `is_online` tinyint NOT NULL DEFAULT 0,
  `connected_players` int NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`, `world_id`)
) ENGINE=InnoDB;