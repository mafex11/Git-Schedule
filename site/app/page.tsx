import { Nav } from "@/components/Nav";
import { Hero } from "@/components/Hero";
import { Features } from "@/components/Features";
import { HowItWorks } from "@/components/HowItWorks";
import { Install } from "@/components/Install";
import { CommandReference } from "@/components/CommandReference";
import { Footer } from "@/components/Footer";
export default function Home() {
  return <main><Nav /><Hero /><Features /><HowItWorks /><Install /><CommandReference /><Footer /></main>;
}
