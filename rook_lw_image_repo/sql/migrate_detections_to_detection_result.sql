-- Migration: Rename detections column and wrap array in DetectionResult structure
-- 1. Renames column from "detections" to "detection"
-- 2. Changes the detection column from an array to an object with "detections" key
--
-- Before: detections = [{"class_id": 1, ...}, ...]
-- After:  detection = {"detections": [{"class_id": 1, ...}, ...]}
--
-- Run with: sqlite3 <database_path> < migrate_detections_to_detection_result.sql

BEGIN TRANSACTION;

-- Step 1: Rename column from detections to detection
ALTER TABLE image_info RENAME COLUMN detections TO detection;

-- Step 2: Update non-empty detection JSON arrays to wrap them in DetectionResult structure
UPDATE image_info 
SET detection = json_object('detections', json(detection))
WHERE detection != '' 
  AND detection IS NOT NULL
  AND json_valid(detection)
  AND json_type(detection) = 'array';

-- Leave empty strings and nulls as-is (they'll be handled by the Rust code)

COMMIT;

-- Verify the migration
SELECT 
    image_id,
    CASE 
        WHEN detection = '' THEN 'empty string'
        WHEN detection IS NULL THEN 'null'
        WHEN json_type(detection) = 'object' THEN 'migrated (object)'
        WHEN json_type(detection) = 'array' THEN 'NOT MIGRATED (still array)'
        ELSE 'unknown'
    END as detection_status,
    substr(detection, 1, 100) as detection_preview
FROM image_info
LIMIT 10;
