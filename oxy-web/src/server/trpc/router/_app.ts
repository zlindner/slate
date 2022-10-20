import { z } from "zod";
import { publicProcedure, router } from "../trpc";
import { pbkdf2, randomBytes } from "crypto";

export const appRouter = router({
  register: publicProcedure
    .input(
      z.object({
        name: z.string(),
        password: z.string()
      })
    )
    .mutation(async ({ input, ctx }) => {
      const hash = await hash_pw(input.password);

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
    })
});

const hash_pw = async (password: string): Promise<string> => {
  return new Promise((resolve, reject) => {
    const salt = randomBytes(256);

    pbkdf2(password, salt, 10000, 64, "sha512", (err, key) => {
      if (err) {
        reject(err);
      } else {
        resolve(key.toString("hex"));
      }
    });
  });
};

export type AppRouter = typeof appRouter;
