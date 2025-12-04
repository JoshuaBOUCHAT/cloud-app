import { useEffect, useState } from "react";
import { useLocation, Link } from "react-router";
import { api } from "../../services/AuthServices";

export default function VerifyPage() {
    const location = useLocation();
    const [status, setStatus] = useState("loading"); // loading | success | error
    const [message, setMessage] = useState("");
    const [resendStatus, setResendStatus] = useState<"idle" | "sending" | "sent" | "failed">("idle");

    useEffect(() => {
        const params = new URLSearchParams(location.search);
        const token = params.get("token");

        if (!token) {
            setStatus("error");
            setMessage("Token introuvable dans l’URL.");
            return;
        }

        api.post("/auth/verify", { key: token })
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

    // --- Fonction pour renvoyer un mail ---
    const handleResendEmail = () => {
        setResendStatus("sending");

        api.post("/auth/verify/resend")
            .then(() => {
                setResendStatus("sent");
            })
            .catch(() => {
                setResendStatus("failed");
            });
    };

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
                            to="/TailAdmin/signin"
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

                        {/* --- Bouton renvoyer email uniquement si non vérifié --- */}
                        <div className="mt-6">
                            {resendStatus === "idle" && (
                                <button
                                    onClick={handleResendEmail}
                                    className="bg-blue-500 hover:bg-blue-600 text-white px-5 py-2 rounded-lg"
                                >
                                    Renvoyer l’email de vérification
                                </button>
                            )}

                            {resendStatus === "sending" && (
                                <p className="text-blue-500 font-medium">Envoi en cours...</p>
                            )}

                            {resendStatus === "sent" && (
                                <p className="text-green-600 font-medium">
                                    Nouvel email envoyé ! Vérifiez votre boîte.
                                </p>
                            )}

                            {resendStatus === "failed" && (
                                <p className="text-red-500 font-medium">
                                    Impossible de renvoyer l’email.
                                </p>
                            )}
                        </div>

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
