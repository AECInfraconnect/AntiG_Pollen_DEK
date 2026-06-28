import type { Meta, StoryObj } from "@storybook/react-vite";
import { Search, ShieldCheck } from "lucide-react";
import { Badge } from "./Badge";
import { Button } from "./Button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./Card";
import { Input } from "./Input";
import { Skeleton } from "./Skeleton";

const meta: Meta = {
  title: "Design System/Primitives",
  parameters: {
    layout: "padded",
  },
};

export default meta;
type Story = StoryObj;

export const Buttons: Story = {
  render: () => (
    <div className="flex flex-wrap gap-3">
      <Button>Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="destructive">Destructive</Button>
      <Button variant="link">Link</Button>
      <Button loading>Checking</Button>
      <Button size="icon" aria-label="Verify policy">
        <ShieldCheck className="h-4 w-4" />
      </Button>
    </div>
  ),
};

export const CardsAndBadges: Story = {
  render: () => (
    <Card className="max-w-md">
      <CardHeader>
        <CardTitle>Observe Coverage</CardTitle>
        <CardDescription>
          Friendly status cards explain what Pollek can see and what still
          needs setup.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex flex-wrap gap-2">
          <Badge variant="ok" dot>
            Observed
          </Badge>
          <Badge variant="info" dot>
            Watching only
          </Badge>
          <Badge variant="degraded" dot>
            Needs setup
          </Badge>
          <Badge variant="outline">Metadata only</Badge>
        </div>
        <Input
          label="Search activity"
          placeholder="Agent, file, website, or command"
          leftIcon={<Search className="h-4 w-4" />}
          hint="Visible labels stay readable in dark and light themes."
        />
      </CardContent>
      <CardFooter>
        <Button variant="outline">Review setup</Button>
        <Button>Open details</Button>
      </CardFooter>
    </Card>
  ),
};

export const LoadingStates: Story = {
  render: () => (
    <div className="grid max-w-2xl gap-4 sm:grid-cols-2">
      <Skeleton className="h-24" />
      <Skeleton className="h-24" />
      <Skeleton className="h-8 sm:col-span-2" />
    </div>
  ),
};
