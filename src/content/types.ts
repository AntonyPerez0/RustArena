export type TestCase = { name: string; stdin: string; expect: string; hidden?: boolean; files?: Record<string, string>; args?: string[]; exit?: number };
export type Rule = { pattern: string; flags?: string; message: string; strings?: boolean; raw?: boolean };

/**
 * stdout: a whole program; each test runs it with some input and compares what it prints.
 * harness: the learner writes functions/types; hidden `#[test]` functions call them.
 * test: the learner writes their own `#[test]` functions, run with `cargo test`.
 */
export type ExerciseMode = "stdout" | "harness" | "test";

export type Exercise = {
  kind: "fill" | "code";
  mode: ExerciseMode;
  seed: string;
  solution: string;
  harness?: string;
  tests: TestCase[];
  checks?: number;
  /** Harness exercises: the name of each check, shown as "what the tests check". */
  checkNames?: string[];
  /** Harness exercises: for each check that must panic (#[should_panic]), the text its message must contain ("" for any panic); null otherwise. */
  checkPanics?: (string | null)[];
  /** Test exercises: how many #[test] functions the learner must write. */
  minTests?: number;
  /** Test exercises: broken copies of the code (find/replace) that the learner's tests must catch. */
  mutants?: { name: string; find: string; replace: string }[];
  hints: string[];
  require: Rule[];
  forbid: Rule[];
};

/** One more exercise on a lesson step's idea, with its own task. */
export type Challenge = Exercise & { task: string };

/**
 * `slug` is the step's permanent address: /learn/<module>/<slug>. `task` is what the first
 * challenge asks for (the step's own exercise); `moreCount` is how many further challenges the
 * step has. Their details load on demand (see challenges.ts).
 */
export type Step = Exercise & { id: string; title: string; text: string; task: string; moreCount?: number; slug: string };

export type Module = {
  id: string;
  title: string;
  phase: string;
  summary: string;
  /** roadmap.sh-style topics this module covers, for the roadmap page. */
  roadmap?: string[];
  steps: Step[];
};

export type Milestone = Exercise & { title: string; text: string };

export type Project = {
  id: string;
  title: string;
  level: string;
  after: string | null;
  summary: string;
  milestones: Milestone[];
};

export type DrillType = "predict" | "fill" | "bug" | "compiles" | "boss" | "choice";

export type Drill = {
  id: string;
  topic: string;
  type: DrillType;
  prompt: string;
  display: string;
  answer: string;
  accept?: string[];
  fix?: string;
  output?: string;
  /** Options for "choice" drills; `answer` is the 1-based index of the right one. */
  choices?: string[];
  why: string;
  /** compiles drills that don't: rustc's error code and first message. */
  error?: string;
  exercise?: Exercise;
  /** The lesson step that teaches what this drill needs; it unlocks in Deathmatch once that step is done. */
  step?: string;
};

export type ProProject = {
  id: string;
  dir: string;
  number: number;
  title: string;
  summary: string;
  hours: number;
  skills: string[];
  readme: string;
};

export type Topic = {
  slug: string;
  title: string;
  description: string;
  modules: string[];
  body: string;
  example: string;
  stdin: string;
  output: string;
};

/** One node of the roadmap page: a roadmap.sh-style topic and the modules that teach it. */
export type RoadmapSection = {
  title: string;
  summary: string;
  items: { title: string; modules?: string[]; pro?: string[]; note?: string; links?: { label: string; url: string }[] }[];
};

export type Content = {
  generatedAt: string;
  modules: Module[];
  projects: Project[];
  drills: Drill[];
  pro: ProProject[];
  /** Placement quiz questions, one per curriculum milestone, in order. */
  placement: (Drill & { module: string })[];
  /** Reference pages at /topics/<slug>. */
  topics: Topic[];
  /** The "Where to go next" page. */
  next: { title: string; description: string; body: string };
  roadmap: RoadmapSection[];
};
