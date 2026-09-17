import React from 'react';
import { Navbar, NavbarSection, Title, Tag, Select, Button, Icon, Stack } from '@pui/components';
import { usePuiTheme } from '@pui/hooks';
import { Sparkles, Activity, ShieldCheck, Sun, Moon } from 'lucide-react';

interface HeaderProps {
  activeTab: string;
  onSelectTab: (tab: string) => void;
}

const BRAND_OPTIONS = [
  { value: 'foundry', label: 'Foundry Blue' },
  { value: 'blueprint', label: 'Blueprint Slate' },
  { value: 'emerald', label: 'Emerald' },
  { value: 'midnight', label: 'Midnight' },
];

export const Header: React.FC<HeaderProps> = () => {
  const { theme, setTheme, brandId, setBrandId } = usePuiTheme();

  return (
    <Navbar sticky variant="solid" height={48}>
      <NavbarSection align="start">
        <Stack direction="row" align="center" gap={2}>
          <Icon icon={Sparkles} size="sm" />
          <Title level={4} size="sm">PHIEGG</Title>
          <Tag intent="primary" minimal round size="sm">AIP COCKPIT</Tag>
        </Stack>
        <Tag intent="success" minimal round size="sm">
          <Icon icon={Activity} size="xs" />
          Resonance: r = 0.984
        </Tag>
        <Tag intent="primary" minimal round size="sm">
          <Icon icon={ShieldCheck} size="xs" />
          Zero-Trust
        </Tag>
      </NavbarSection>

      <NavbarSection align="end">
        <Stack direction="row" align="center" gap={2}>
          <Select
            size="sm"
            value={brandId}
            options={BRAND_OPTIONS}
            onChange={(e) => setBrandId(e.target.value)}
          />
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
            icon={<Icon icon={theme === 'dark' ? Sun : Moon} size="xs" />}
          />
        </Stack>
      </NavbarSection>
    </Navbar>
  );
};
