import { router } from "../trpc";
import { registerRouter } from "./register";

export const appRouter = router({
  register: registerRouter
});

export type AppRouter = typeof appRouter;
