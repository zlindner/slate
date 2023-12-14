CREATE TABLE `keymaps` (
  `id` int NOT NULL AUTO_INCREMENT,
  `character_id` int NOT NULL,
  `key` int NOT NULL,
  `type_` int NOT NULL,
  `action` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;