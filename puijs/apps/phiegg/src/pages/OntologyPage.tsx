import React from 'react';
import { Page, Stack, Grid, Col, Card, CardHeader, CardBody, Title, Text, Badge, Button, Icon } from '@pui/components';
import { useStore } from '@pui/store';
import { ontologyStore } from '../state/ontologyStore';
import { OntologyTable } from '../components/Ontology/OntologyTable';
import { OntologyDrawer } from '../components/Ontology/OntologyDrawer';
import { Layers } from 'lucide-react';

export const OntologyPage: React.FC = () => {
  const { objectTypes, linkTypes, actions, selectedTypeId, selectedInstance, selectType, selectInstance } = useStore(ontologyStore);
  const currentType = objectTypes.find((t) => t.id === selectedTypeId) || objectTypes[0];

  return (
    <Page>
      <Stack direction="column" gap={4}>
        <Stack direction="row" justify="between" align="center" wrap>
          <Stack direction="column" gap={1}>
            <Title level={2} size="xl">POntology Graph & Object Explorer</Title>
            <Text size="md" intent="secondary">Palantir Workshop-symmetrical entity grid, link traversals, and action mutations.</Text>
          </Stack>
          <Stack direction="row" gap={2} wrap>
            {objectTypes.map((t) => (
              <Button
                key={t.id}
                variant={t.id === selectedTypeId ? 'primary' : 'ghost'}
                size="sm"
                onClick={() => selectType(t.id)}
              >
                {t.displayName} ({t.count})
              </Button>
            ))}
          </Stack>
        </Stack>

        <Grid gap={3} xs={1} sm={1} md={1} lg={12}>
          <Col span={8} xs={12} sm={12} md={12} lg={8}>
            <OntologyTable currentType={currentType} onSelectInstance={selectInstance} />
          </Col>

          <Col span={4} xs={12} sm={12} md={12} lg={4}>
            <Card variant="flat">
              <CardHeader>
                <Stack direction="row" align="center" gap={2}>
                  <Icon icon={Layers} size="sm" />
                  <Title level={4} size="md">Ontology Link Graph</Title>
                </Stack>
              </CardHeader>
              <CardBody>
                <Stack direction="column" gap={2}>
                  {linkTypes.map((link) => (
                    <Card key={link.id} variant="compact">
                      <CardBody>
                        <Stack direction="row" justify="between" align="center">
                          <Text size="sm" intent="primary" weight="medium">{link.sourceType} → {link.targetType}</Text>
                          <Badge variant="neutral" size="sm">{link.cardinality}</Badge>
                        </Stack>
                      </CardBody>
                    </Card>
                  ))}
                </Stack>
              </CardBody>
            </Card>
          </Col>
        </Grid>
      </Stack>

      <OntologyDrawer
        selectedInstance={selectedInstance}
        actions={actions}
        onClose={() => selectInstance(null)}
      />
    </Page>
  );
};
