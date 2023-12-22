CREATE TABLE `items` (
  `id` int NOT NULL AUTO_INCREMENT,
  `item_id` int NOT NULL,
  `character_id` int NOT NULL,
  `inventory_type` enum('Equip','Use','Setup','Etc','Cash') NOT NULL,
  `position` int NOT NULL,
  `amount` int NOT NULL,
  `owner` varchar(13) NOT NULL DEFAULT '',
  `flag` int NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;