export interface PropertyDef {
  name: string;
  type: 'string' | 'integer' | 'double' | 'boolean' | 'timestamp' | 'attachment' | 'timeseries' | 'ciphertext';
  required: boolean;
  indexed?: boolean;
}

export interface ObjectTypeDef {
  id: string;
  apiName: string;
  displayName: string;
  description: string;
  primaryKey: string;
  icon: string;
  properties: PropertyDef[];
  count: number;
}

export interface LinkTypeDef {
  id: string;
  apiName: string;
  displayName: string;
  sourceType: string;
  targetType: string;
  cardinality: 'ONE_TO_ONE' | 'ONE_TO_MANY' | 'MANY_TO_ONE' | 'MANY_TO_MANY';
}

export interface ActionDef {
  id: string;
  apiName: string;
  displayName: string;
  targetObjectType: string;
  parameters: Array<{ name: string; type: string; required: boolean }>;
  morphismType: 'CREATE' | 'MODIFY' | 'DELETE' | 'DISPATCH';
}
