CREATE TABLE `accounts` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(13) UNIQUE NOT NULL,
  `password` varchar(256) NOT NULL,
  `pin` varchar(10) NOT NULL DEFAULT '',
  `pic` varchar(26) NOT NULL DEFAULT '',
  `state` enum('LoggedIn','Transitioning','LoggedOut') NOT NULL DEFAULT 'LoggedOut',
  `banned` tinyint(1) NOT NULL DEFAULT 0,
  `accepted_tos` tinyint(1) NOT NULL DEFAULT 0,
  `last_login` datetime DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `gender` tinyint(1) NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;