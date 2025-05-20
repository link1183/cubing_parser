USE solves;

CREATE TABLE IF NOT EXISTS `event` (
    `id` int AUTO_INCREMENT NOT NULL UNIQUE,
    `event_name` varchar(10) NOT NULL UNIQUE,
    PRIMARY KEY (`id`)
);

CREATE TABLE IF NOT EXISTS `solve` (
    `id` int AUTO_INCREMENT NOT NULL UNIQUE,
    `event_name` varchar(10) NOT NULL,
    `time` int NOT NULL,
    `scramble` varchar(250) NOT NULL,
    `date` datetime NOT NULL,
    `session_name` varchar(100) NOT NULL,
    `penalty` int NOT NULL DEFAULT '0',
    `comment` text,
    PRIMARY KEY (`id`)
);

ALTER TABLE `solve` ADD CONSTRAINT `solve_fk1` FOREIGN KEY (`event_name`) REFERENCES `event`(`event_name`);
