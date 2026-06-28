import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { Badge } from "./Badge";
import { Button } from "./Button";
import { Card, CardContent, CardHeader, CardTitle } from "./Card";
import { Dialog } from "./Dialog";
import { Input } from "./Input";
import { Skeleton } from "./Skeleton";

describe("UI primitives", () => {
  it("renders button variants with accessible busy state", () => {
    const html = renderToStaticMarkup(
      <Button variant="primary" loading>
        Save
      </Button>,
    );

    expect(html).toContain("bg-primary");
    expect(html).toContain("aria-busy=\"true\"");
    expect(html).toContain("disabled=\"\"");
  });

  it("renders card, badge, input, and skeleton with token-based classes", () => {
    const html = renderToStaticMarkup(
      <Card>
        <CardHeader>
          <CardTitle>Agent status</CardTitle>
        </CardHeader>
        <CardContent>
          <Badge variant="ok" dot>
            Observing
          </Badge>
          <Input label="Agent name" hint="Visible to the user" />
          <Skeleton className="h-4 w-12" />
        </CardContent>
      </Card>,
    );

    expect(html).toContain("Agent status");
    expect(html).toContain("Observing");
    expect(html).toContain("Agent name");
    expect(html).toContain("bg-card");
    expect(html).toContain("bg-muted");
  });

  it("renders dialog with the expected accessibility contract", () => {
    const html = renderToStaticMarkup(
      <Dialog
        open
        title="Confirm change"
        description="Review before applying."
        onClose={() => {}}
      >
        <button type="button">Keep editing</button>
      </Dialog>,
    );

    expect(html).toContain("role=\"dialog\"");
    expect(html).toContain("aria-modal=\"true\"");
    expect(html).toContain("Confirm change");
    expect(html).toContain("Review before applying.");
    expect(html).toContain("Close dialog");
  });
});
