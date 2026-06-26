import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Kiva Werewolf Director",
  description: "Werewolf match director and replay recorder",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN">
      <body>{children}</body>
    </html>
  );
}
