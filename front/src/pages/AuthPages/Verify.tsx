import { useEffect, useState } from "react";
import { useLocation, Link } from "react-router";
import { api } from "../../services/AuthServices";

export default function VerifyPage() {
    const location = useLocation();
    const [status, setStatus] = useState("loading"); // loading | success | error
    const [message, setMessage] = useState("");

    useEffect(() => {
        const params = new URLSearchParams(location.search);
        const token = params.get("token");

        if (!token) {
            setStatus("error");
            setMessage("Token introuvable dans l’URL.");
            return;
        }

        api.post("/verify", { key: token })
            .then((res) => {
                setStatus("success");
                setMessage(res.data.message || "Votre compte est maintenant vérifié !");
            })
            .catch((err) => {
                console.error(err);

                if (err.response?.data?.message) {
                    setMessage(err.response.data.message);
                } else {
                    setMessage("La vérification a échoué.");
                }

                setStatus("error");
            });
    }, [location]);

    return (
        <div className="min-h-screen flex items-center justify-center bg-gray-100 px-4">
            <div className="bg-white shadow-lg rounded-xl p-8 max-w-md w-full text-center">

                {/* LOADING */}
                {status === "loading" && (
                    <>
                        <div className="animate-spin h-10 w-10 border-4 border-blue-400 border-t-transparent mx-auto rounded-full"></div>
                        <h1 className="text-xl font-semibold mt-4">Vérification en cours...</h1>
                        <p className="text-gray-500 mt-2">
                            Merci de patienter pendant la validation de votre compte.
                        </p>
                    </>
                )}

                {/* SUCCESS */}
                {status === "success" && (
                    <>
                        <div className="text-green-500 text-5xl mb-4">✓</div>
                        <h1 className="text-2xl font-bold">Compte vérifié !</h1>
                        <p className="text-gray-600 mt-2">{message}</p>

                        <Link
                            to="/login"
                            className="mt-6 inline-block bg-green-500 hover:bg-green-600 text-white px-5 py-2 rounded-lg"
                        >
                            Se connecter
                        </Link>
                    </>
                )}

                {/* ERROR */}
                {status === "error" && (
                    <>
                        <div className="text-red-500 text-5xl mb-4">✕</div>
                        <h1 className="text-2xl font-bold">Échec de vérification</h1>
                        <p className="text-gray-600 mt-2">{message}</p>

                        <Link
                            to="/"
                            className="mt-6 inline-block bg-red-500 hover:bg-red-600 text-white px-5 py-2 rounded-lg"
                        >
                            Retour à l'accueil
                        </Link>
                    </>
                )}
            </div>
        </div>
    );
}
