CREATE TABLE `keymaps` (
  `id` int NOT NULL AUTO_INCREMENT,
  `character_id` int NOT NULL,
  `key_id` int NOT NULL,
  `key_type` int NOT NULL,
  `action` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;