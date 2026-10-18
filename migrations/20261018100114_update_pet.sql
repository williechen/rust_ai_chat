-- Add migration script here
ALTER TABLE pet_owner
    MODIFY COLUMN pet_owner_id TEXT not null;

ALTER TABLE pet_profile
    DROP COLUMN owner_id;
    ADD COLUMN pet_owner_id TEXT not null;
    
ALTER TABLE pet_extra
    DROP COLUMN owner_id;
    ADD COLUMN pet_owner_id TEXT not null;

ALTER TABLE pet_activity
    DROP COLUMN owner_id;
    ADD COLUMN pet_owner_id TEXT not null;

ALTER TABLE pet_study
    DROP COLUMN owner_id;
    ADD COLUMN pet_owner_id TEXT not null;

ALTER TABLE pet_life_log
    DROP COLUMN owner_id;
    ADD COLUMN pet_owner_id TEXT not null;

ALTER TABLE pet_profile
    DROP CONSTRAINT pet_profile_owner_id_fkey,
    ADD CONSTRAINT pet_profile_owner_id_fkey FOREIGN KEY (pet_owner_id) REFERENCES pet_owner(id);

ALTER TABLE pet_extra
    DROP CONSTRAINT pet_extra_owner_id_fkey,
    ADD CONSTRAINT pet_extra_owner_id_fkey FOREIGN KEY (pet_owner_id) REFERENCES pet_owner(id);

ALTER TABLE pet_activity
    DROP CONSTRAINT pet_activity_owner_id_fkey,
    ADD CONSTRAINT pet_activity_owner_id_fkey FOREIGN KEY (pet_owner_id) REFERENCES pet_owner(id);

ALTER TABLE pet_study
    DROP CONSTRAINT pet_study_owner_id_fkey,
    ADD CONSTRAINT pet_study_owner_id_fkey FOREIGN KEY (pet_owner_id) REFERENCES pet_owner(id);

ALTER TABLE pet_life_log
    DROP CONSTRAINT pet_life_log_owner_id_fkey,
    ADD CONSTRAINT pet_life_log_owner_id_fkey FOREIGN KEY (pet_owner_id) REFERENCES pet_owner(id);

