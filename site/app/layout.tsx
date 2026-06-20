import type { Metadata } from "next";
import localFont from "next/font/local";
import { JetBrains_Mono } from "next/font/google";
import "./globals.css";

const satoshi = localFont({
  variable: "--font-sans",
  display: "swap",
  src: [
    { path: "../public/fonts/Satoshi-Regular.woff2", weight: "400", style: "normal" },
    { path: "../public/fonts/Satoshi-Medium.woff2", weight: "500", style: "normal" },
    { path: "../public/fonts/Satoshi-Bold.woff2", weight: "700", style: "normal" },
    { path: "../public/fonts/Satoshi-Black.woff2", weight: "900", style: "normal" },
  ],
});

const jetbrains = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["400", "500"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "git-schedule — delayed send for your git commits",
  description:
    "Schedule git commits and pull requests for later. Write code at midnight, let it commit at 9 AM — locally or in the cloud. A Rust CLI.",
  openGraph: {
    title: "git-schedule — delayed send for your git commits",
    description: "Schedule git commits and pull requests for later.",
    type: "website",
  },
  twitter: { card: "summary_large_image" },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${satoshi.variable} ${jetbrains.variable}`}>
      <body>{children}</body>
    </html>
  );
}
