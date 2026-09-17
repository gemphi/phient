import React from 'react';
import { Card, CardHeader, CardBody, Title, Tag, Badge, Button, Icon, Table, Thead, Tbody, Tr, Th, Td, Stack } from '@pui/components';
import { Database } from 'lucide-react';
import { ObjectTypeDef } from '../../types/ontology';

const MOCK_FLIGHT_ROWS = [
  { id: 'FL-101', tail: 'N402AA', origin: 'JFK', dest: 'LHR', status: 'EN_ROUTE', alt: 34000, fuel: '78%' },
  { id: 'FL-102', tail: 'N814AA', origin: 'SFO', dest: 'HND', status: 'EN_ROUTE', alt: 38000, fuel: '64%' },
  { id: 'FL-103', tail: 'N109AA', origin: 'ORD', dest: 'CDG', status: 'BOARDING', alt: 0, fuel: '100%' },
  { id: 'FL-104', tail: 'N902AA', origin: 'LAX', dest: 'SYD', status: 'EN_ROUTE', alt: 36000, fuel: '82%' },
  { id: 'FL-105', tail: 'N301AA', origin: 'MIA', dest: 'GRU', status: 'DELAYED', alt: 0, fuel: '90%' },
];

interface OntologyTableProps {
  currentType: ObjectTypeDef;
  onSelectInstance: (inst: Record<string, any>) => void;
}

export const OntologyTable: React.FC<OntologyTableProps> = ({ currentType, onSelectInstance }) => {
  return (
    <Card variant="default">
      <CardHeader action={<Tag intent="primary" minimal round>{currentType.count} Instances</Tag>}>
        <Stack direction="row" align="center" gap={2}>
          <Icon icon={Database} size="sm" />
          <Title level={4} size="sm">ObjectSet: {currentType.displayName}</Title>
        </Stack>
      </CardHeader>
      <CardBody>
        <Table>
          <Thead>
            <Tr>
              <Th>Primary Key</Th>
              <Th>Tail #</Th>
              <Th>Origin</Th>
              <Th>Destination</Th>
              <Th>Status</Th>
              <Th>Altitude</Th>
              <Th>Fuel</Th>
              <Th>Action</Th>
            </Tr>
          </Thead>
          <Tbody>
            {MOCK_FLIGHT_ROWS.map((row) => (
              <Tr key={row.id}>
                <Td><Badge variant="primary">{row.id}</Badge></Td>
                <Td>{row.tail}</Td>
                <Td>{row.origin}</Td>
                <Td>{row.dest}</Td>
                <Td>
                  <Tag intent={row.status === 'EN_ROUTE' ? 'success' : row.status === 'DELAYED' ? 'error' : 'warning'} minimal round size="sm">
                    {row.status}
                  </Tag>
                </Td>
                <Td>{row.alt.toLocaleString()} ft</Td>
                <Td>{row.fuel}</Td>
                <Td>
                  <Button size="sm" variant="ghost" onClick={() => onSelectInstance(row)}>
                    Inspect
                  </Button>
                </Td>
              </Tr>
            ))}
          </Tbody>
        </Table>
      </CardBody>
    </Card>
  );
};
