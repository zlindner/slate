CREATE TABLE `skills` (
  `id` int NOT NULL,
  `character_id` int NOT NULL,
  `level` int NOT NULL,
  `mastery` int NOT NULL,
  `expiration` bigint NOT NULL,
  PRIMARY KEY (`id`, `character_id`)
) ENGINE=InnoDB;