import React from 'react';
import { Drawer, Stack, Text, Title, Card, CardBody, Button, Icon } from '@pui/components';
import { Zap } from 'lucide-react';
import { ActionDef } from '../../types/ontology';

interface OntologyDrawerProps {
  selectedInstance: Record<string, any> | null;
  actions: ActionDef[];
  onClose: () => void;
}

export const OntologyDrawer: React.FC<OntologyDrawerProps> = ({ selectedInstance, actions, onClose }) => {
  return (
    <Drawer
      isOpen={Boolean(selectedInstance)}
      onClose={onClose}
      title={`Object Detail: ${selectedInstance?.id || ''}`}
    >
      <Stack direction="column" gap={3}>
        <Text size="md" intent="secondary">Inspecting live object properties and linked actions.</Text>
        <Card variant="compact">
          <CardBody>
            <Stack direction="column" gap={2}>
              <Text size="sm" weight="medium">Tail Number: {selectedInstance?.tail}</Text>
              <Text size="sm" weight="medium">Route: {selectedInstance?.origin} → {selectedInstance?.dest}</Text>
              <Text size="sm" weight="medium">Status: {selectedInstance?.status}</Text>
              <Text size="sm" weight="medium">Altitude: {selectedInstance?.alt} ft</Text>
            </Stack>
          </CardBody>
        </Card>
        <Title level={4} size="md">Available Morphism Actions</Title>
        {actions.map((act) => (
          <Button key={act.id} variant="primary" size="md" icon={<Icon icon={Zap} size="sm" />}>
            Execute: {act.displayName}
          </Button>
        ))}
      </Stack>
    </Drawer>
  );
};
