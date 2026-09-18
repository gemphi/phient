'use client';

import React from 'react';
import { Stack, Tag, Text, MenuItem } from '@pui/components';
import { DOCS_NAV_GROUPS, NavItem } from '../content/navigation';

interface AppSidebarProps {
  currentPath: string;
  onNavigate: (path: string) => void;
}

export const AppSidebar: React.FC<AppSidebarProps> = ({ currentPath, onNavigate }) => {
  return (
    <Stack direction="column" gap={6}>
      {DOCS_NAV_GROUPS.map((group) => (
        <Stack key={group.id} direction="column" gap={2}>
          <Text size="xs" weight="semibold" intent="muted">{group.title}</Text>
          <Stack direction="column" gap={1}>
            {group.items.map((item: NavItem) => {
              const active = currentPath === item.path;
              return (
                <MenuItem
                  key={item.id}
                  active={active}
                  onClick={() => onNavigate(item.path)}
                >
                  <Stack direction="row" align="center" justify="between" gap={2}>
                    <Text size="sm">{item.label}</Text>
                    {item.badge && (
                      <Tag intent={active ? 'primary' : 'none'} minimal size="sm">
                        {item.badge}
                      </Tag>
                    )}
                  </Stack>
                </MenuItem>
              );
            })}
          </Stack>
        </Stack>
      ))}
    </Stack>
  );
};
