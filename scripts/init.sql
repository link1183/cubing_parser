CREATE DATABASE IF NOT EXISTS solves;
USE solves;

CREATE TABLE IF NOT EXISTS `event` (
    `id` int AUTO_INCREMENT NOT NULL UNIQUE,
    `event_name` varchar(10) NOT NULL UNIQUE,
    PRIMARY KEY (`id`)
);

CREATE TABLE IF NOT EXISTS `solve` (
    `id` int AUTO_INCREMENT NOT NULL UNIQUE,
    `event_name` int NOT NULL,
    `time` int NOT NULL,
    `scramble` varchar(250) NOT NULL,
    `date` datetime NOT NULL,
    `session_name` varchar(100) NOT NULL,
    `penalty` int NOT NULL DEFAULT '0',
    `comment` text,
    PRIMARY KEY (`id`)
);

INSERT INTO event (event_name) VALUES 
('222'),
('333'),
('444'),
('555'),
('666'),
('777'),
('sq1'),
('skewb'),
('clock'),
('pyra'),
('mega'),
('3bld'),
('fc'),
('4bld'),
('5bld'),
('multi'),
('oh');
