import type { Metadata } from 'next';

import './globals.css';

export const metadata: Metadata = {
  title: 'TlantiCAD Studio',
  description: 'Offline dental CAD, DICOM, AI and Tauri workstation.',
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="es" suppressHydrationWarning>
      <body>{children}</body>
    </html>
  );
}
