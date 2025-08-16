-- phpMyAdmin SQL Dump
-- version 5.2.1
-- https://www.phpmyadmin.net/
--
-- Host: 127.0.0.1
-- Generation Time: Aug 16, 2025 at 08:47 AM
-- Server version: 10.4.32-MariaDB
-- PHP Version: 8.0.30

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `job_finder`
--

-- --------------------------------------------------------

--
-- Table structure for table `jobs`
--

CREATE TABLE `jobs` (
  `job_id` varchar(64) NOT NULL,
  `request_id` varchar(64) DEFAULT NULL,
  `search_query` varchar(255) DEFAULT NULL,
  `employer_name` varchar(255) DEFAULT NULL,
  `employer_logo` varchar(1024) DEFAULT NULL,
  `employer_website` varchar(1024) DEFAULT NULL,
  `employer_company_type` varchar(255) DEFAULT NULL,
  `employer_linkedin` varchar(1024) DEFAULT NULL,
  `job_publisher` varchar(255) DEFAULT NULL,
  `job_employment_type` varchar(64) DEFAULT NULL,
  `job_employment_type_text` varchar(64) DEFAULT NULL,
  `job_employment_types_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_employment_types_json`)),
  `job_title` varchar(512) DEFAULT NULL,
  `job_apply_link` varchar(1024) DEFAULT NULL,
  `job_apply_is_direct` tinyint(1) DEFAULT NULL,
  `job_apply_quality_score` decimal(6,2) DEFAULT NULL,
  `job_description` longtext DEFAULT NULL,
  `job_is_remote` tinyint(1) DEFAULT NULL,
  `job_posted_human_readable` varchar(64) DEFAULT NULL,
  `job_posted_at_timestamp` bigint(20) DEFAULT NULL,
  `job_posted_at_datetime_utc` datetime DEFAULT NULL,
  `job_location` varchar(255) DEFAULT NULL,
  `job_city` varchar(128) DEFAULT NULL,
  `job_state` varchar(128) DEFAULT NULL,
  `job_country` varchar(16) DEFAULT NULL,
  `job_latitude` decimal(10,7) DEFAULT NULL,
  `job_longitude` decimal(10,7) DEFAULT NULL,
  `job_benefits_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_benefits_json`)),
  `job_google_link` varchar(1024) DEFAULT NULL,
  `job_offer_expiration_datetime_utc` datetime DEFAULT NULL,
  `job_offer_expiration_timestamp` bigint(20) DEFAULT NULL,
  `no_experience_required` tinyint(1) DEFAULT NULL,
  `required_experience_in_months` int(11) DEFAULT NULL,
  `experience_mentioned` tinyint(1) DEFAULT NULL,
  `experience_preferred` tinyint(1) DEFAULT NULL,
  `job_salary_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_salary_json`)),
  `job_min_salary` decimal(18,2) DEFAULT NULL,
  `job_max_salary` decimal(18,2) DEFAULT NULL,
  `job_salary_currency` varchar(8) DEFAULT NULL,
  `job_salary_period` varchar(32) DEFAULT NULL,
  `job_highlights_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`job_highlights_json`)),
  `job_job_title` varchar(255) DEFAULT NULL,
  `job_posting_language` varchar(16) DEFAULT NULL,
  `job_onet_soc` varchar(32) DEFAULT NULL,
  `job_onet_job_zone` varchar(32) DEFAULT NULL,
  `raw_json` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_bin DEFAULT NULL CHECK (json_valid(`raw_json`)),
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `updated_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `matching_analysis` text NOT NULL,
  `cover_letter` text DEFAULT NULL,
  `isdelete` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- --------------------------------------------------------

--
-- Table structure for table `job_apply_options`
--

CREATE TABLE `job_apply_options` (
  `id` bigint(20) UNSIGNED NOT NULL,
  `job_id` varchar(64) NOT NULL,
  `publisher` varchar(255) DEFAULT NULL,
  `apply_link` varchar(1024) DEFAULT NULL,
  `is_direct` tinyint(1) DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data for table `job_apply_options`
--

--
-- Table structure for table `myresume`
--

CREATE TABLE `myresume` (
  `id` int(11) NOT NULL,
  `description` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data for table `myresume`
--



--
-- Indexes for dumped tables
--

--
-- Indexes for table `jobs`
--
ALTER TABLE `jobs`
  ADD PRIMARY KEY (`job_id`),
  ADD KEY `idx_title` (`job_title`(191)),
  ADD KEY `idx_employer` (`employer_name`(191)),
  ADD KEY `idx_city` (`job_city`),
  ADD KEY `idx_country` (`job_country`);

--
-- Indexes for table `job_apply_options`
--
ALTER TABLE `job_apply_options`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `uniq_job_link` (`job_id`,`apply_link`(191));

--
-- Indexes for table `myresume`
--
ALTER TABLE `myresume`
  ADD PRIMARY KEY (`id`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `job_apply_options`
--
ALTER TABLE `job_apply_options`
  MODIFY `id` bigint(20) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=794;

--
-- AUTO_INCREMENT for table `myresume`
--
ALTER TABLE `myresume`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=2;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `job_apply_options`
--
ALTER TABLE `job_apply_options`
  ADD CONSTRAINT `fk_apply_job` FOREIGN KEY (`job_id`) REFERENCES `jobs` (`job_id`) ON DELETE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
