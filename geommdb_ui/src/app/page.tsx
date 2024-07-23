import ConnectionCard from "@/components/connection-card";
import { ModeToggle } from "@/components/toggle-theme";
import { Button } from "@/components/ui/button";
import Image from "next/image";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="grid grid-cols-3 gap-4">
        <ConnectionCard />
        <ConnectionCard />
        <ConnectionCard />
        <ConnectionCard />
      </div>
    </main>
  );
}
