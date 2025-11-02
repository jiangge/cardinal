export type SlabIndex = number & { readonly __slabIndex: unique symbol };

export const toSlabIndex = (value: number): SlabIndex => value as SlabIndex;

export const toSlabIndexArray = (values: number[]): SlabIndex[] =>
  values.map((value) => value as SlabIndex);
