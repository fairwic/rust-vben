ALTER TABLE admin_users
  ADD COLUMN IF NOT EXISTS "desc" TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS timezone TEXT NOT NULL DEFAULT 'Asia/Shanghai';

INSERT INTO menus (id, pid, name, path, auth_code, component, redirect, menu_type, status, meta)
VALUES
  ('10', NULL, 'Dashboard', '/dashboard', NULL, NULL, '/analytics', 'catalog', 1, '{"order":-1,"title":"page.dashboard.title"}'::jsonb),
  ('100', '10', 'Analytics', '/analytics', NULL, '/dashboard/analytics/index', NULL, 'menu', 1, '{"affixTab":true,"title":"page.dashboard.analytics"}'::jsonb),
  ('20', NULL, 'Demos', '/demos', NULL, NULL, '/demos/access', 'catalog', 1, '{"icon":"ic:baseline-view-in-ar","keepAlive":true,"order":1000,"title":"demos.title"}'::jsonb),
  ('210', '20', 'AccessDemos', '/demos/access', NULL, NULL, '/demos/access/page-control', 'catalog', 1, '{"icon":"mdi:cloud-key-outline","title":"demos.access.backendPermissions"}'::jsonb),
  ('211', '210', 'AccessPageControlDemo', '/demos/access/page-control', NULL, '/demos/access/index', NULL, 'menu', 1, '{"icon":"mdi:page-previous-outline","title":"demos.access.pageAccess"}'::jsonb),
  ('212', '210', 'AccessButtonControlDemo', '/demos/access/button-control', NULL, '/demos/access/button-control', NULL, 'menu', 1, '{"icon":"mdi:button-cursor","title":"demos.access.buttonControl"}'::jsonb),
  ('213', '210', 'AccessSuperVisibleDemo', '/demos/access/super-visible', NULL, '/demos/access/super-visible', NULL, 'menu', 1, '{"icon":"mdi:button-cursor","title":"demos.access.superVisible"}'::jsonb),
  ('214', '210', 'AccessAdminVisibleDemo', '/demos/access/admin-visible', NULL, '/demos/access/admin-visible', NULL, 'menu', 1, '{"icon":"mdi:button-cursor","title":"demos.access.adminVisible"}'::jsonb),
  ('215', '210', 'AccessUserVisibleDemo', '/demos/access/user-visible', NULL, '/demos/access/user-visible', NULL, 'menu', 1, '{"icon":"mdi:button-cursor","title":"demos.access.userVisible"}'::jsonb),
  ('21201', '212', 'AccessSuperCode', '', 'AC_100100', NULL, NULL, 'button', 1, '{"title":"demos.access.superVisible"}'::jsonb),
  ('21202', '212', 'AccessAdminCode', '', 'AC_100030', NULL, NULL, 'button', 1, '{"title":"demos.access.adminVisible"}'::jsonb),
  ('21203', '212', 'AccessUserCode', '', 'AC_1000001', NULL, NULL, 'button', 1, '{"title":"demos.access.userVisible"}'::jsonb)
ON CONFLICT (id) DO NOTHING;

UPDATE menus
SET pid = '10',
    path = '/workspace',
    component = '/dashboard/workspace/index',
    redirect = NULL,
    menu_type = 'menu',
    status = 1,
    meta = '{"title":"page.dashboard.workspace"}'::jsonb,
    updated_at = NOW()
WHERE id = '1';

UPDATE roles
SET permissions = (
  SELECT to_jsonb(COALESCE(array_agg(DISTINCT permission ORDER BY permission), ARRAY[]::text[]))
  FROM (
    SELECT jsonb_array_elements_text(roles.permissions) AS permission
    UNION
    SELECT unnest(ARRAY[
      '10','100','1',
      '20','210','211','212','213',
      '21201','21202'
    ])
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
      '10','100','1',
      '20','210','211','212','214',
      '21202'
    ])
  ) AS merged_permissions
)
WHERE id = 'role-admin';

UPDATE roles
SET permissions = (
  SELECT to_jsonb(COALESCE(array_agg(DISTINCT permission ORDER BY permission), ARRAY[]::text[]))
  FROM (
    SELECT jsonb_array_elements_text(roles.permissions) AS permission
    UNION
    SELECT unnest(ARRAY[
      '10','100','1',
      '20','210','211','212','215',
      '21203'
    ])
  ) AS merged_permissions
)
WHERE id = 'role-user';
