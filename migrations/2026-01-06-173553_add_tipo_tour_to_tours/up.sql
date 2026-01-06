-- Add tipo_tour column to tours table
-- Optional VARCHAR field with empty string default

ALTER TABLE tours 
ADD COLUMN tipo_tour VARCHAR(100) DEFAULT '';
