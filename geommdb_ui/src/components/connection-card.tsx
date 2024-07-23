/**
 * v0 by Vercel.
 * @see https://v0.dev/t/zCP3j9rTQHk
 * Documentation: https://v0.dev/docs#integrating-generated-code-into-your-nextjs-app
 */
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export default function ConnectionCard() {
  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>Connect to GEOMMDB</CardTitle>
        <CardDescription>
          Connect your application to a GEOMMDB instance.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4">
          <div className="grid gap-2">
            <Label htmlFor="host">Host</Label>
            <Input id="host" placeholder="Enter GEOMMDB host" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="port">Port</Label>
            <Input id="port" type="number" placeholder="Enter GEOMMDB port" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              type="password"
              placeholder="Enter GEOMMDB password"
            />
          </div>
          <Button type="submit" className="w-full">
            Connect
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
