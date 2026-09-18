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
import { Search, Sun, Moon, Sparkles } from 'lucide-react';

export const CockpitHeader: React.FC = () => {
  const { theme, setTheme, brandId, setBrandId, brands, isDark } = usePuiTheme();

  return (
    <Navbar sticky variant="solid" height={56}>
      <NavbarSection align="start">
        <Stack direction="row" align="center" gap={3}>
          <Icon icon={Sparkles} size="md" tone="primary" />
          <Title level={4} size="sm">PUI Enterprise Cockpit</Title>
          <Tag intent="primary" minimal>Palantir Foundry Parity</Tag>
        </Stack>
      </NavbarSection>

      <NavbarSection align="end">
        <Stack direction="row" align="center" gap={3}>
          <Input
            placeholder="Search entities, agents, axioms..."
            icon={<Icon icon={Search} size="xs" tone="muted" />}
          />
          <Button
            variant="outline"
            size="sm"
            onClick={() => setTheme(isDark ? 'light' : 'dark')}
            icon={<Icon icon={isDark ? Sun : Moon} size="xs" />}
          >
            {isDark ? 'Light' : 'Dark'}
          </Button>
          <Select
            value={brandId}
            onChange={(e) => setBrandId(e.target.value)}
            options={brands.map((b) => ({ value: b.id, label: b.name }))}
          />
        </Stack>
      </NavbarSection>
    </Navbar>
  );
};
