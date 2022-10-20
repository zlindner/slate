import type { NextPage } from "next";
import Head from "next/head";
import { trpc } from "../utils/trpc";
import { useForm } from "react-hook-form";

type RegisterFormData = {
  name: string;
  password: string;
}

const Home: NextPage = () => {
  const registerMutation = trpc.register.useMutation();
  const { register, handleSubmit, formState: { errors } } = useForm<RegisterFormData>();

  let nameClass = "relative block w-full appearance-none rounded-none rounded-t-md border border-gray-300 px-3 py-2 text-gray-900 placeholder-gray-500 focus:z-10 focus:outline-none sm:text-sm ";

  if (errors.name) {
    nameClass += "focus:border-red-500 focus:ring-red-500";
  } else {
    nameClass += "focus:border-blue-500 focus:ring-blue-500";
  }

  let passwordClass = "mb-8 relative block w-full appearance-none rounded-none rounded-b-md border border-gray-300 px-3 py-2 text-gray-900 placeholder-gray-500 focus:z-10 focus:border-blue-500 focus:outline-none focus:ring-blue-500 sm:text-sm ";

  if (errors.password) {
    passwordClass += "focus:border-red-500 focus:ring-red-500";
  } else {
    passwordClass += "focus:border-blue-500 focus:ring-blue-500";
  }

  return (
    <>
      <Head>
        <title>oxy</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main className="flex min-h-full items-center justify-center py-12 px-4 sm:px-6 lg:px-8 bg-gray-50">
        <div className="w-full max-w-sm space-y-8">
          <form className="space-y-6" onSubmit={handleSubmit(registerMutation.mutate)}>
            <div className="-space-y-px rounded-md shadow-sm">
              <div>
                <label htmlFor="name" className="sr-only">Name</label>
                <input
                  {...register("name", { required: true, minLength: 4, maxLength: 13 })}
                  placeholder="Name"
                  className={nameClass}
                />
              </div>

              <div>
                <label htmlFor="password" className="sr-only">Password</label>
                <input
                  {...register("password", { required: true, minLength: 8, maxLength: 64 })}
                  type="password"
                  placeholder="Password"
                  className={passwordClass}
                />
              </div>

              <button
                type="submit"
                className="group relative flex w-full justify-center rounded-md border border-transparent bg-blue-600 py-2 px-4 text-sm font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
              >
                Register
              </button>
            </div>
          </form>
        </div>
      </main>
    </>
  );
};

export default Home;
