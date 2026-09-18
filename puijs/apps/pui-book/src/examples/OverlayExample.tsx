'use client';

import React, { useState } from 'react';
import { Button, Dialog, Drawer, Stack, Text, ProgressBar } from '@pui/components';

export const OverlayExample: React.FC = () => {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [drawerOpen, setDrawerOpen] = useState(false);

  return (
    <Stack direction="column" gap={3}>
      <Stack direction="row" gap={2}>
        <Button variant="primary" onClick={() => setDialogOpen(true)}>Open Modal Dialog</Button>
        <Button variant="outline" onClick={() => setDrawerOpen(true)}>Open Slideout Drawer</Button>
      </Stack>

      <Dialog isOpen={dialogOpen} onClose={() => setDialogOpen(false)} title="Ontology Inspector">
        <Stack direction="column" gap={3} padding="16px 0">
          <Text>Inspecting entity schema for FinanceTransaction with strict typing invariants.</Text>
          <ProgressBar value={0.85} intent="primary" />
        </Stack>
      </Dialog>

      <Drawer isOpen={drawerOpen} onClose={() => setDrawerOpen(false)} title="Live Telemetry" position="right">
        <Stack direction="column" gap={2} padding={20}>
          <Text variant="sm" color="secondary">Streaming telemetry data from Phiano phase manifold.</Text>
        </Stack>
      </Drawer>
    </Stack>
  );
};
