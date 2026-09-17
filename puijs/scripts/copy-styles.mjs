import { cpSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const srcDir = join(rootDir, 'src');
const distDir = join(rootDir, 'dist');

cpSync(srcDir, distDir, {
  recursive: true,
  filter: (source, _destination) => {
    if (source === srcDir) return true;
    const name = source.replace(/\\/g, '/');
    if (name.endsWith('.ts') || name.endsWith('.tsx') || name.endsWith('.js') || name.endsWith('.d.ts')) return false;
    return !source.split(/[\\/]/).includes('node_modules');
  },
});

const styleTypePath = join(distDir, 'styles', 'core.scss.d.ts');
mkdirSync(dirname(styleTypePath), { recursive: true });
writeFileSync(styleTypePath, 'declare const styles: string;\nexport default styles;\n');
