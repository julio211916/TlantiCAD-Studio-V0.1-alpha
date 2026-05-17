"use client";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const tabs = [
    { title: "Product", value: "product" },
    { title: "Services", value: "services" },
    { title: "Playground", value: "playground" },
    { title: "Content", value: "content" },
    { title: "Random", value: "random" },
] as const;

export default function TabsDemo() {
    return (
        <Tabs defaultValue={tabs[0].value} className="mx-auto my-20 flex w-full max-w-5xl flex-col gap-6 [perspective:1000px]">
            <TabsList>
                {tabs.map((tab) => (
                    <TabsTrigger key={tab.value} value={tab.value}>
                        {tab.title}
                    </TabsTrigger>
                ))}
            </TabsList>

            {tabs.map((tab) => (
                <TabsContent key={tab.value} value={tab.value}>
                    <div className="relative min-h-[20rem] overflow-hidden rounded-[1.75rem] bg-[linear-gradient(135deg,#40205f_0%,#120b27_100%)] p-6 text-white shadow-2xl md:min-h-[32rem] md:p-10">
                        <p className="text-xl font-bold md:text-4xl">{tab.title} tab</p>
                        <DummyContent />
                    </div>
                </TabsContent>
            ))}
        </Tabs>
    );
}

const DummyContent = () => {
    return (
        <div className="absolute inset-x-6 bottom-6 top-24 grid gap-3 md:grid-cols-[1.4fr_0.8fr]">
            <div className="rounded-[1.4rem] border border-white/15 bg-white/8 p-4 backdrop-blur-sm">
                <div className="grid h-full gap-3 rounded-[1.2rem] border border-white/10 bg-black/15 p-4">
                    <div className="h-8 w-36 rounded-full bg-white/20" />
                    <div className="h-4 w-2/3 rounded-full bg-white/10" />
                    <div className="grid flex-1 gap-3 md:grid-cols-2">
                        <div className="rounded-[1.1rem] bg-white/10" />
                        <div className="rounded-[1.1rem] bg-white/5" />
                    </div>
                </div>
            </div>
            <div className="grid gap-3">
                <div className="rounded-[1.2rem] bg-white/10" />
                <div className="rounded-[1.2rem] bg-white/5" />
            </div>
        </div>
    );
};
