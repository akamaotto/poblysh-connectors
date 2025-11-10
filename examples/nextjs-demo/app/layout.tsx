import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { DemoProvider } from "@/lib/demo/state";
import { DemoModeIndicator } from "@/components/DemoModeIndicator";
import { DemoNavbar } from "@/components/demo/DemoNavbar";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Poblysh Connectors Demo",
  description: "Mock demonstration of Poblysh Connectors integration flow",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased bg-white min-h-screen`}
      >
        <DemoProvider>
          <DemoModeIndicator />
          <DemoNavbar />
          <main className="min-h-screen pt-16">
            {children}
          </main>
        </DemoProvider>
      </body>
    </html>
  );
}
