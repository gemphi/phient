import React from 'react';
import { Page, Stack, Grid, Col, Card, CardHeader, CardBody, Title, Text, Tag, Badge, Icon, Table, Thead, Tbody, Tr, Th, Td, ProgressBar } from '@pui/components';
import { MapPin, Compass, Globe } from 'lucide-react';

const SPATIAL_POINTS = [
  { id: 'SPT-01', lat: 37.7749, lon: -122.4194, label: 'SFO Terminal Hub', cluster: 'Cluster #A', similarity: 0.992 },
  { id: 'SPT-02', lat: 40.7128, lon: -74.0060, label: 'JFK Radar Station', cluster: 'Cluster #A', similarity: 0.984 },
  { id: 'SPT-03', lat: 51.5074, lon: -0.1278, label: 'LHR Control Tower', cluster: 'Cluster #B', similarity: 0.976 },
  { id: 'SPT-04', lat: 35.6762, lon: 139.6503, label: 'HND Pacific Gate', cluster: 'Cluster #C', similarity: 0.965 },
];

export const SpatialPage: React.FC = () => {
  return (
    <Page>
      <Stack direction="column" gap={4}>
        <Stack direction="row" justify="between" align="center" wrap>
          <Stack direction="column" gap={1}>
            <Title level={2} size="xl">PhiOra Spatial & Vector Embeddings Explorer</Title>
            <Text size="md" intent="secondary">HNSW nearest-neighbor vector similarity clustering and GeoSpatial coordinate indexing.</Text>
          </Stack>
          <Tag intent="primary" minimal round size="md">
            <Icon icon={Globe} size="xs" />
            Active CRS: EPSG:4326 (WGS84)
          </Tag>
        </Stack>

        <Grid gap={3} xs={1} sm={1} md={1} lg={12}>
          <Col span={7} xs={12} sm={12} md={12} lg={7}>
            <Card variant="default">
              <CardHeader action={<Tag intent="success" minimal round size="md">HNSW Indexed</Tag>}>
                <Stack direction="row" align="center" gap={2}>
                  <Icon icon={Compass} size="sm" />
                  <Title level={4} size="md">Nearest-Neighbor Vector Clusters</Title>
                </Stack>
              </CardHeader>
              <CardBody>
                <Table>
                  <Thead>
                    <Tr>
                      <Th>Vector ID</Th>
                      <Th>Label</Th>
                      <Th>Cluster</Th>
                      <Th>Coordinates (Lat, Lon)</Th>
                      <Th>Cosine Similarity</Th>
                    </Tr>
                  </Thead>
                  <Tbody>
                    {SPATIAL_POINTS.map((pt) => (
                      <Tr key={pt.id}>
                        <Td><Badge variant="primary">{pt.id}</Badge></Td>
                        <Td><Text size="sm">{pt.label}</Text></Td>
                        <Td><Tag intent="primary" minimal round size="sm">{pt.cluster}</Tag></Td>
                        <Td><Text size="sm" mono>{pt.lat.toFixed(4)}, {pt.lon.toFixed(4)}</Text></Td>
                        <Td>
                          <Stack direction="column" gap={1}>
                            <Text size="sm" weight="medium">{pt.similarity.toFixed(3)}</Text>
                            <ProgressBar value={pt.similarity} intent="success" size="sm" />
                          </Stack>
                        </Td>
                      </Tr>
                    ))}
                  </Tbody>
                </Table>
              </CardBody>
            </Card>
          </Col>

          <Col span={5} xs={12} sm={12} md={12} lg={5}>
            <Card variant="flat">
              <CardHeader>
                <Stack direction="row" align="center" gap={2}>
                  <Icon icon={MapPin} size="sm" />
                  <Title level={4} size="md">Git KV Content-Addressed Store</Title>
                </Stack>
              </CardHeader>
              <CardBody>
                <Stack direction="column" gap={2}>
                  <Card variant="compact">
                    <CardBody>
                      <Stack direction="column" gap={1}>
                        <Text size="sm" intent="primary" weight="medium">Root Tree Hash: 0x8849...210f</Text>
                        <Text size="sm" intent="muted">Commit ID: c_99182 · Author: phigit</Text>
                        <Text size="sm" intent="secondary">Contains 4,820 coordinate blobs and HNSW adjacency lists.</Text>
                      </Stack>
                    </CardBody>
                  </Card>
                </Stack>
              </CardBody>
            </Card>
          </Col>
        </Grid>
      </Stack>
    </Page>
  );
};
