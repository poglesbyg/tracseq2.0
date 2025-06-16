-- Fix storage_movement_history table to allow NULL to_location_id for removal operations
-- This allows us to record when samples are removed from storage entirely

ALTER TABLE storage_movement_history 
ALTER COLUMN to_location_id DROP NOT NULL;

-- Update the foreign key constraint to allow NULL values
ALTER TABLE storage_movement_history 
DROP CONSTRAINT storage_movement_history_to_location_id_fkey;

ALTER TABLE storage_movement_history 
ADD CONSTRAINT storage_movement_history_to_location_id_fkey 
FOREIGN KEY (to_location_id) REFERENCES storage_locations(id); 
