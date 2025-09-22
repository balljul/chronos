-- Migration: Insert 20 sample users
-- Created: 2025-09-22

INSERT INTO users (email, password_hash, first_name, last_name, is_verified, is_active) VALUES
-- Verified and active users
('alice.johnson@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/Zex6bE3eJ8KL9.rTW', 'Alice', 'Johnson', TRUE, TRUE),
('bob.smith@example.com', '$2b$12$EixZxYPK8/2QHBSj7DBzKOe6Sv9EELn.W7VxGZjK2hGsY9VlXNpqO', 'Bob', 'Smith', TRUE, TRUE),
('carol.davis@example.com', '$2b$12$VGz8cJKpRx4LQ6tGKw3P.uB5Rv9EILm.V7VxGZjH2hFsX8UkYOqrN', 'Carol', 'Davis', TRUE, TRUE),
('david.wilson@example.com', '$2b$12$HQn9dKLqSy4MT7uHL.4Q.uC6Sx0FJMo.W8WyH0kI3hGtZ9VmZPrsP', 'David', 'Wilson', TRUE, TRUE),
('eva.brown@example.com', '$2b$12$IRP0eLMrTz5NU8vIM.5R.vD7Ty1GKNp.X9XzIAkJ4hHu10WnAPstQ', 'Eva', 'Brown', TRUE, TRUE),

-- Mix of verified/unverified users
('frank.miller@example.com', '$2b$12$JSQ1fMNsU06OV9wJN.6S.wE8Uz2HLOq.Y0Y0JAkK5hIv21XoBQuR', 'Frank', 'Miller', FALSE, TRUE),
('grace.taylor@example.com', '$2b$12$KTR2gOOtV17PW0xKO.7T.xF9V03IMPr.Z1Z1KBkL6hJw32YpCRvsS', 'Grace', 'Taylor', TRUE, TRUE),
('henry.anderson@example.com', '$2b$12$LUS3hPPuW28QX1yLP.8U.yG0W14JNQs.A2A2LClM7hKx43ZqDSwtT', 'Henry', 'Anderson', FALSE, TRUE),
('iris.thomas@example.com', '$2b$12$MVT4iQQvX39RY2zMQ.9V.zH1X25KORt.B3B3MDmN8hLy54ArETxuU', 'Iris', 'Thomas', TRUE, TRUE),
('jack.jackson@example.com', '$2b$12$NWU5jRRwY40SZ30NR.0W.0I2Y36LPSu.C4C4NEnO9hMz65BsFUyvV', 'Jack', 'Jackson', FALSE, TRUE),

-- Some users with only first names
('kevin@example.com', '$2b$12$OXV6kSSxZ51T641OS.1X.1J3Z47MQTv.D5D5OFoP0hN067CtGVzwW', 'Kevin', NULL, TRUE, TRUE),
('linda@example.com', '$2b$12$PYW7lTTyA62U652PT.2Y.2K4A58NRUw.E6E6PGpQ1hO178DuHW0xX', 'Linda', NULL, FALSE, TRUE),
('mike@example.com', '$2b$12$QZX8mUUzB73V663QU.3Z.3L5B69OSVx.F7F7QHqR2hP289EvIX1yY', 'Mike', NULL, TRUE, TRUE),

-- Some inactive users
('nancy.white@example.com', '$2b$12$RA9nVVA0C84W674RV.40.4M6C70PTWy.G8G8RIsS3hQ390FwJY2zZ', 'Nancy', 'White', TRUE, FALSE),
('oscar.green@example.com', '$2b$12$SB0oWWB1D95X785X.51.5N7D81QUXz.H9H9SJtT4hR402GxKZ310', 'Oscar', 'Green', FALSE, FALSE),

-- Professional email domains
('sarah.connor@techcorp.com', '$2b$12$TC1pXXC2E06Y896TY.62.6O8E92RVY0.I0I0TKuU5hS513HyL420', 'Sarah', 'Connor', TRUE, TRUE),
('john.doe@startup.io', '$2b$12$UD2qYYD3F17Z907UZ.73.7P9F03SWZ1.J1J1ULvV6hT624IzM531', 'John', 'Doe', TRUE, TRUE),
('jane.admin@company.org', '$2b$12$VE3rZZE4G28a108V8.84.8Q0G14TXA2.K2K2VMwW7hU735J0N642', 'Jane', 'Admin', TRUE, TRUE),

-- International users
('pavel.novak@example.cz', '$2b$12$WF4saF5H39b219W9.95.9R1H25UYB3.L3L3WNxX8hV846K1O753', 'Pavel', 'Novák', TRUE, TRUE),
('maria.garcia@example.es', '$2b$12$XG5tbG6I40c320X0.06.0S2I36VZC4.M4M4XOyY9hW957L2P864', 'María', 'García', FALSE, TRUE);
