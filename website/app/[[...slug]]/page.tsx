// app/[[...slug]]/page.tsx
import { source } from "@/lib/source";
import {
  DocsPage,
  DocsBody,
  DocsDescription,
  DocsTitle,
} from "fumadocs-ui/page";
import { notFound } from "next/navigation";
import defaultMdxComponents from "fumadocs-ui/mdx";
import type { MDXComponents } from "mdx/types";
import { metadataImage } from "@/lib/metadata-image";
import { createMetadata } from "@/lib/metadata";
import { Metadata } from "next/types";

interface PageProps {
  params: Promise<{ slug?: string[] }>;
}

const mdxComponents = {
  ...defaultMdxComponents,
} as MDXComponents;

export default async function Page(props: PageProps) {
  const { slug = [] } = await props.params;

  // Get the page data
  const page = source.getPage(slug);
  if (!page) notFound();

  const path = `content/docs/${page.file.path}`;

  const MDX = page.data.body;

  const Content = (
    <DocsBody>
      <MDX components={mdxComponents} />
    </DocsBody>
  );

  return (
    <DocsPage
      toc={page.data.toc}
      full={page.data.full}
      editOnGithub={{
        owner: "MetisProtocol",
        repo: "metis-sdk",
        sha: "main",
        path,
      }}
    >
      <DocsTitle>{page.data.title}</DocsTitle>
      <DocsDescription>{page.data.description}</DocsDescription>
      {Content}
    </DocsPage>
  );
}

export async function generateMetadata(props: {
  params: Promise<{ slug: string[] }>;
}): Promise<Metadata> {
  const params = await props.params;
  const page = source.getPage(params.slug);

  if (!page) notFound();

  const description = page.data.description;

  return createMetadata(
    metadataImage.withImage(page.slugs, {
      title: page.data.title,
      description,
      openGraph: {
        url: `/${page.slugs.join("/")}`,
      },
    })
  );
}

export function generateStaticParams(): { slug: string[] }[] {
  return source.generateParams();
}
