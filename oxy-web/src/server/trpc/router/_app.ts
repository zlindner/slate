import { z } from "zod";
import { publicProcedure, router } from "../trpc";
import argon2 from "argon2";

export const appRouter = router({
  register: publicProcedure
    .input(
      z.object({
        name: z.string(),
        password: z.string()
      })
    )
    .mutation(async ({ input, ctx }) => {
      try {
        const hash = await argon2.hash(input.password);

        const account = await ctx.prisma.account.create({
          data: {
            name: input.name,
            password: hash,
            pin: "",
            pic: "",
            gender: 0
          }
        });

        return { success: true, account };
      } catch (err) {
        return { success: false };
      }
    })
});

export type AppRouter = typeof appRouter;
