'use client';

import React from 'react';
import {
  Button,
  Input,
  Select,
  Stack,
  Tag,
  Title,
  Icon,
  Navbar,
  NavbarSection,
  usePuiTheme,
} from '@pui/components';
import { Search, Sun, Moon, Sparkles, Sliders } from 'lucide-react';

interface AppHeaderProps {
  onOpenInspector: () => void;
  searchQuery: string;
  onSearchChange: (q: string) => void;
}

export const AppHeader: React.FC<AppHeaderProps> = ({
  onOpenInspector,
  searchQuery,
  onSearchChange,
}) => {
  const { setTheme, brandId, setBrandId, brands, isDark } = usePuiTheme();

  return (
    <Navbar sticky variant="solid" height={56}>
      <NavbarSection align="start">
        <Stack direction="row" align="center" gap={3}>
          <Icon icon={Sparkles} size="md" tone="primary" />
          <Title level={4} size="sm">PUI Documentation & Workbench</Title>
          <Tag intent="primary" minimal round>puijs.com</Tag>
        </Stack>
      </NavbarSection>

      <NavbarSection align="end">
        <Stack direction="row" align="center" gap={3}>
          <Stack width={260}>
            <Input
              placeholder="Search components, tokens, APIs..."
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              icon={<Icon icon={Search} size="xs" tone="muted" />}
            />
          </Stack>

          <Button
            variant="outline"
            size="sm"
            onClick={() => setTheme(isDark ? 'light' : 'dark')}
            icon={<Icon icon={isDark ? Sun : Moon} size="xs" />}
          >
            {isDark ? 'Light' : 'Dark'}
          </Button>

          <Stack width={140}>
            <Select
              value={brandId}
              onChange={(e) => setBrandId(e.target.value)}
              options={brands.map((b) => ({ value: b.id, label: b.name }))}
            />
          </Stack>

          <Button
            variant="secondary"
            size="sm"
            icon={<Icon icon={Sliders} size="xs" />}
            onClick={onOpenInspector}
          >
            Inspector
          </Button>
        </Stack>
      </NavbarSection>
    </Navbar>
  );
};
