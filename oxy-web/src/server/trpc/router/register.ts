import * as trpc from "@trpc/server";
import { z } from "zod";
import { prisma } from "../../db/client";

export const registerRouter = trpc
  .router()
  .mutation("register", {
    input: z.object({
      name: z.string(),
      password: z.string()
    }),
    async resolve({ input }) {
      const password = input.password; // TODO hash!!!

      const account = await prisma.account.create({
        data: {
          name: input.name,
          password: password
        }
      });

      return { success: true, account };
    }
  });

// export type definition of API
export type RegisterRouter = typeof registerRouter;