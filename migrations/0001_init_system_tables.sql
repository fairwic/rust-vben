CREATE TABLE IF NOT EXISTS roles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
  remark TEXT NOT NULL DEFAULT '',
  status SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS menus (
  id TEXT PRIMARY KEY,
  pid TEXT NULL REFERENCES menus(id) ON DELETE CASCADE,
  name TEXT NOT NULL UNIQUE,
  path TEXT NOT NULL DEFAULT '',
  auth_code TEXT NULL,
  component TEXT NULL,
  redirect TEXT NULL,
  menu_type TEXT NOT NULL,
  status SMALLINT NOT NULL,
  meta JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_menus_path_unique
  ON menus(path)
  WHERE path <> '';

CREATE TABLE IF NOT EXISTS depts (
  id TEXT PRIMARY KEY,
  pid TEXT NULL REFERENCES depts(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  remark TEXT NOT NULL DEFAULT '',
  status SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO roles (id, name, permissions, remark, status)
VALUES
  ('role-super', '超级管理员', '["200","20001","20002","20003","201","20101","20102","20103","202","20201","20202","20203"]'::jsonb, '拥有系统全部菜单权限', 1),
  ('role-admin', '管理员', '["200","201","202"]'::jsonb, '拥有系统管理访问权限', 1),
  ('role-user', '普通用户', '[]'::jsonb, '没有系统管理权限', 0)
ON CONFLICT (id) DO NOTHING;

INSERT INTO menus (id, pid, name, path, auth_code, component, redirect, menu_type, status, meta)
VALUES
  ('1', NULL, 'Workspace', '/workspace', NULL, '/dashboard/workspace/index', NULL, 'menu', 1, '{"icon":"carbon:workspace","title":"page.dashboard.workspace","affixTab":true,"order":0}'::jsonb),
  ('2', NULL, 'System', '/system', NULL, NULL, NULL, 'catalog', 1, '{"icon":"carbon:settings","order":9997,"title":"system.title","badge":"new","badgeType":"normal","badgeVariants":"primary"}'::jsonb),
  ('200', '2', 'SystemRole', '/system/role', 'System:Role:List', '/system/role/list', NULL, 'menu', 1, '{"icon":"mdi:account-group","title":"system.role.title"}'::jsonb),
  ('20001', '200', 'SystemRoleCreate', '', 'System:Role:Create', NULL, NULL, 'button', 1, '{"title":"common.create"}'::jsonb),
  ('20002', '200', 'SystemRoleEdit', '', 'System:Role:Edit', NULL, NULL, 'button', 1, '{"title":"common.edit"}'::jsonb),
  ('20003', '200', 'SystemRoleDelete', '', 'System:Role:Delete', NULL, NULL, 'button', 1, '{"title":"common.delete"}'::jsonb),
  ('201', '2', 'SystemMenu', '/system/menu', 'System:Menu:List', '/system/menu/list', NULL, 'menu', 1, '{"icon":"carbon:menu","title":"system.menu.title"}'::jsonb),
  ('20101', '201', 'SystemMenuCreate', '', 'System:Menu:Create', NULL, NULL, 'button', 1, '{"title":"common.create"}'::jsonb),
  ('20102', '201', 'SystemMenuEdit', '', 'System:Menu:Edit', NULL, NULL, 'button', 1, '{"title":"common.edit"}'::jsonb),
  ('20103', '201', 'SystemMenuDelete', '', 'System:Menu:Delete', NULL, NULL, 'button', 1, '{"title":"common.delete"}'::jsonb),
  ('202', '2', 'SystemDept', '/system/dept', 'System:Dept:List', '/system/dept/list', NULL, 'menu', 1, '{"icon":"carbon:container-services","title":"system.dept.title"}'::jsonb),
  ('20201', '202', 'SystemDeptCreate', '', 'System:Dept:Create', NULL, NULL, 'button', 1, '{"title":"common.create"}'::jsonb),
  ('20202', '202', 'SystemDeptEdit', '', 'System:Dept:Edit', NULL, NULL, 'button', 1, '{"title":"common.edit"}'::jsonb),
  ('20203', '202', 'SystemDeptDelete', '', 'System:Dept:Delete', NULL, NULL, 'button', 1, '{"title":"common.delete"}'::jsonb)
ON CONFLICT (id) DO NOTHING;

INSERT INTO depts (id, pid, name, remark, status)
VALUES
  ('1', NULL, '总部', '总部节点', 1),
  ('2', '1', '研发中心', '系统研发部门', 1),
  ('3', '1', '运营中心', '系统运营部门', 1)
ON CONFLICT (id) DO NOTHING;
