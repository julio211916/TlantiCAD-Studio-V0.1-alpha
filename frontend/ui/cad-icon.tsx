import {
  Box,
  Circle,
  Cone,
  Cylinder,
  DraftingCompass,
  Layers,
  Minus,
  Move3D,
  Ruler,
  Scissors,
  Shapes,
  Square,
  type LucideIcon,
} from 'lucide-react';

const iconMap: Record<string, LucideIcon> = {
  box: Box,
  boolean: Shapes,
  chamfer: Scissors,
  cone: Cone,
  cylinder: Cylinder,
  default: Circle,
  fillet: DraftingCompass,
  measure: Ruler,
  move: Move3D,
  shell: Layers,
  sphere: Circle,
  subtract: Minus,
  torus: Circle,
  union: Shapes,
  wedge: Square,
};

export interface CadIconProps {
  icon?: string;
  name?: string;
  className?: string;
}

export function CadIcon({ icon, name, className }: CadIconProps) {
  const key = (icon ?? name ?? 'default').toLowerCase();
  const Icon = iconMap[key] ?? iconMap.default;

  return <Icon className={className} aria-hidden="true" />;
}
