-- Revert: Remove test users

DELETE FROM users WHERE username IN ('admin', 'subadmin', 'usuario', 'viewer');
