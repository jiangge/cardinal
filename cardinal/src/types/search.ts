export type SearchResultMetadata = {
  type?: number;
  size?: number;
  mtime?: number;
  ctime?: number;
};

export type SearchResultRecord = {
  path?: string;
  metadata?: SearchResultMetadata;
  size?: number;
  mtime?: number;
  ctime?: number;
  icon?: string;
};

export type SearchResultItem = string | SearchResultRecord;

export type NodeInfoResponse = {
  path: string;
  icon?: string | null;
  metadata?: SearchResultMetadata;
};
