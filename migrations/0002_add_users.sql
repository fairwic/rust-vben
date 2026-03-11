CREATE TABLE IF NOT EXISTS admin_users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  real_name TEXT NOT NULL,
  avatar TEXT NOT NULL DEFAULT '',
  home_path TEXT NOT NULL DEFAULT '/analytics',
  email TEXT NOT NULL DEFAULT '',
  phone TEXT NOT NULL DEFAULT '',
  dept_id TEXT NULL REFERENCES depts(id) ON DELETE SET NULL,
  remark TEXT NOT NULL DEFAULT '',
  status SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_users_email_unique
  ON admin_users(email)
  WHERE email <> '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_users_phone_unique
  ON admin_users(phone)
  WHERE phone <> '';

CREATE TABLE IF NOT EXISTS user_roles (
  user_id TEXT NOT NULL REFERENCES admin_users(id) ON DELETE CASCADE,
  role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, role_id)
);

INSERT INTO menus (id, pid, name, path, auth_code, component, redirect, menu_type, status, meta)
VALUES
  ('203', '2', 'SystemUser', '/system/user', 'System:User:List', '/system/user/list', NULL, 'menu', 1, '{"icon":"lucide:user","title":"system.user.title"}'::jsonb),
  ('20301', '203', 'SystemUserCreate', '', 'System:User:Create', NULL, NULL, 'button', 1, '{"title":"common.create"}'::jsonb),
  ('20302', '203', 'SystemUserEdit', '', 'System:User:Edit', NULL, NULL, 'button', 1, '{"title":"common.edit"}'::jsonb),
  ('20303', '203', 'SystemUserDelete', '', 'System:User:Delete', NULL, NULL, 'button', 1, '{"title":"common.delete"}'::jsonb)
ON CONFLICT (id) DO NOTHING;

UPDATE roles
SET permissions = (
  SELECT to_jsonb(COALESCE(array_agg(DISTINCT permission ORDER BY permission), ARRAY[]::text[]))
  FROM (
    SELECT jsonb_array_elements_text(roles.permissions) AS permission
    UNION
    SELECT unnest(ARRAY['203', '20301', '20302', '20303'])
  ) AS merged_permissions
)
WHERE id = 'role-super';

UPDATE roles
SET permissions = (
  SELECT to_jsonb(COALESCE(array_agg(DISTINCT permission ORDER BY permission), ARRAY[]::text[]))
  FROM (
    SELECT jsonb_array_elements_text(roles.permissions) AS permission
    UNION
    SELECT unnest(ARRAY[
      '20001', '20002', '20003',
      '20101', '20102', '20103',
      '20201', '20202', '20203',
      '203', '20301', '20302', '20303'
    ])
  ) AS merged_permissions
)
WHERE id = 'role-admin';

INSERT INTO admin_users (id, username, password, real_name, avatar, home_path, email, phone, dept_id, remark, status)
VALUES
  ('user-0', 'vben', '123456', 'Vben', '', '/analytics', 'vben@example.com', '13800000000', '2', '超级管理员账号', 1),
  ('user-1', 'admin', '123456', 'Admin', '', '/workspace', 'admin@example.com', '13800000001', '2', '管理员账号', 1),
  ('user-2', 'jack', '123456', 'Jack', '', '/analytics', 'jack@example.com', '13800000002', '3', '普通用户账号', 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
VALUES
  ('user-0', 'role-super'),
  ('user-1', 'role-admin'),
  ('user-2', 'role-user')
ON CONFLICT (user_id, role_id) DO NOTHING;
