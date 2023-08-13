CREATE TABLE `accounts` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(13) NOT NULL,
  `password` varchar(13) NOT NULL,
  `pin` varchar(10) NOT NULL,
  `pic` varchar(26) NOT NULL,
  `state` enum('LoggedIn','Transitioning','LoggedOut') NOT NULL,
  `banned` tinyint(1) NOT NULL,
  `accepted_tos` tinyint(1) NOT NULL,
  `last_login` datetime NOT NULL,
  `gender` tinyint NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;