import React from 'react';
import { Page, Stack, Card, CardHeader, CardBody, Title, Text, Tag, Badge, Button, Icon, Table, Thead, Tbody, Tr, Th, Td } from '@pui/components';
import { useStore } from '@pui/store';
import { streamStore } from '../state/streamStore';
import { Radio, Play, Pause, Trash2, ShieldCheck, Activity } from 'lucide-react';

const TAXONOMIES = ['ALL', 'AGENT_STATUS', 'ONTOLOGY_MUTATION', 'TRANSACTION_COMMITTED', 'AUDIT_EVENT', 'TELEMETRY_PULSE'] as const;

export const StreamsPage: React.FC = () => {
  const { packets, isPaused, selectedTaxonomy, togglePause, setTaxonomy, clearPackets } = useStore(streamStore);

  const filteredPackets = selectedTaxonomy === 'ALL'
    ? packets
    : packets.filter((p) => p.taxonomy === selectedTaxonomy);

  return (
    <Page>
      <Stack direction="column" gap={4}>
        <Stack direction="row" justify="between" align="center">
          <Stack direction="column" gap={1}>
            <Title level={2} size="xl">PhiBus Live Event Streaming Monitor</Title>
            <Text size="md" intent="secondary">Real-time Server-Sent Events (SSE) packet tailing, Merkle signatures, and event replay.</Text>
          </Stack>
          <Stack direction="row" gap={2} align="center">
            <Button
              size="sm"
              variant={isPaused ? 'danger' : 'primary'}
              onClick={togglePause}
              icon={<Icon icon={isPaused ? Play : Pause} size="xs" />}
            >
              {isPaused ? 'Resume Stream' : 'Pause Stream'}
            </Button>
            <Button size="sm" variant="ghost" onClick={clearPackets} icon={<Icon icon={Trash2} size="xs" />}>
              Clear
            </Button>
          </Stack>
        </Stack>

        <Stack direction="row" gap={2} align="center">
          {TAXONOMIES.map((t) => (
            <Button
              key={t}
              size="sm"
              variant={selectedTaxonomy === t ? 'primary' : 'ghost'}
              onClick={() => setTaxonomy(t as any)}
            >
              {t}
            </Button>
          ))}
        </Stack>

        <Card variant="default">
          <CardHeader
            action={
              <Stack direction="row" align="center" gap={2}>
                <Tag intent="success" minimal round size="md">
                  <Icon icon={Activity} size="xs" />
                  Stream Rate: 14.8k evt/s
                </Tag>
              </Stack>
            }
          >
            <Stack direction="row" align="center" gap={2}>
              <Icon icon={Radio} size="sm" />
              <Title level={4} size="md">Live Packet Waterfall ({filteredPackets.length})</Title>
            </Stack>
          </CardHeader>
          <CardBody>
            <Table>
              <Thead>
                <Tr>
                  <Th>Sequence</Th>
                  <Th>Timestamp</Th>
                  <Th>Taxonomy</Th>
                  <Th>Source Agent</Th>
                  <Th>Payload Preview</Th>
                  <Th>Merkle Proof</Th>
                </Tr>
              </Thead>
              <Tbody>
                {filteredPackets.map((pkt) => (
                  <Tr key={pkt.id}>
                    <Td><Badge variant="neutral">#{pkt.sequence}</Badge></Td>
                    <Td><Text size="sm">{pkt.timestamp}</Text></Td>
                    <Td><Tag intent="primary" minimal round size="sm">{pkt.taxonomy}</Tag></Td>
                    <Td><Badge variant="primary">{pkt.source}</Badge></Td>
                    <Td><Text size="sm" mono>{JSON.stringify(pkt.payload)}</Text></Td>
                    <Td>
                      <Stack direction="row" align="center" gap={1}>
                        <Icon icon={ShieldCheck} size="xs" />
                        <Text size="xs" intent="muted">{pkt.merkleSignature}</Text>
                      </Stack>
                    </Td>
                  </Tr>
                ))}
              </Tbody>
            </Table>
          </CardBody>
        </Card>
      </Stack>
    </Page>
  );
};
